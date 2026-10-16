use crate::applets::archival::common::*;
use crate::applets::archival::inflate::inflate_all;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

struct ZipEntry {
    name: Vec<u8>,
    method: u16,
    csize: u32,
    usize_: u32,
    local_off: u32,
}

fn find_eocd(data: &[u8]) -> Option<usize> {
    if data.len() < 22 {
        return None;
    }
    let start = data.len().saturating_sub(22 + 65535);
    let mut i = data.len() - 22;
    loop {
        if data[i] == b'P' && data[i + 1] == b'K' && data[i + 2] == 5 && data[i + 3] == 6 {
            return Some(i);
        }
        if i == start {
            return None;
        }
        i -= 1;
    }
}

fn parse_zip(data: &[u8]) -> std::io::Result<Vec<ZipEntry>> {
    let e = find_eocd(data)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "cannot find EOCD"))?;
    let count = u16le(data, e + 10);
    let cd_off = u32le(data, e + 16) as usize;
    let mut entries = Vec::new();
    let mut p = cd_off;
    for _ in 0..count {
        if p + 46 > data.len() || &data[p..p + 4] != b"PK\x01\x02" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad central directory",
            ));
        }
        let method = u16le(data, p + 10) as u16;
        let csize = u32le(data, p + 20);
        let usize_ = u32le(data, p + 24);
        let nlen = u16le(data, p + 28);
        let elen = u16le(data, p + 30);
        let clen = u16le(data, p + 32);
        let local_off = u32le(data, p + 42);
        if p + 46 + nlen > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad central name",
            ));
        }
        entries.push(ZipEntry {
            name: data[p + 46..p + 46 + nlen].to_vec(),
            method,
            csize,
            usize_,
            local_off,
        });
        p += 46 + nlen + elen + clen;
    }
    Ok(entries)
}

fn zip_raw_data<'a>(data: &'a [u8], e: &ZipEntry) -> std::io::Result<&'a [u8]> {
    let o = e.local_off as usize;
    if o + 30 > data.len() || &data[o..o + 4] != b"PK\x03\x04" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad local header",
        ));
    }
    let nlen = u16le(data, o + 26);
    let elen = u16le(data, o + 28);
    let start = o + 30 + nlen + elen;
    let end = start + e.csize as usize;
    if end > data.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated entry data",
        ));
    }
    Ok(&data[start..end])
}

fn zip_entry_data(data: &[u8], e: &ZipEntry) -> std::io::Result<Vec<u8>> {
    let raw = zip_raw_data(data, e)?;
    match e.method {
        0 => Ok(raw.to_vec()),
        8 => {
            let out = inflate_all(raw)?;
            if out.len() as u32 != e.usize_ {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "size mismatch after inflate",
                ));
            }
            Ok(out)
        }
        m => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("unsupported zip method {m}"),
        )),
    }
}

pub struct UnzipApplet;
impl Applet for UnzipApplet {
    fn name(&self) -> &'static str {
        "unzip"
    }
    fn description(&self) -> &'static str {
        "Extract/list stored+deflated zip archives (-l/-p/-o/-n/-d subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut list = false;
        let mut to_pipe = false;
        let mut overwrite = false;
        let mut never = false;
        let mut dest: Option<PathBuf> = None;
        let mut positional: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-l" {
                list = true;
            } else if b == b"-p" {
                to_pipe = true;
            } else if b == b"-o" {
                overwrite = true;
            } else if b == b"-n" {
                never = true;
            } else if b == b"-q" || b == b"-v" {
            } else if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("unzip: -d requires an argument");
                    return Ok(1);
                }
                dest = Some(PathBuf::from(&args[i]));
            } else {
                positional.push(&args[i]);
            }
            i += 1;
        }
        if positional.is_empty() {
            eprintln!("unzip: missing zipfile");
            return Ok(1);
        }
        let data = match std::fs::read(Path::new(&positional[0])) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("unzip: {}: {e}", positional[0].to_string_lossy());
                return Ok(1);
            }
        };
        let entries = match parse_zip(&data) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("unzip: bad archive: {e}");
                return Ok(1);
            }
        };
        let filter: Vec<&[u8]> = positional[1..].iter().map(|a| ab(a)).collect();
        let wanted = |n: &[u8]| filter.is_empty() || filter.contains(&n);
        let out = std::io::stdout();
        let mut lock = out.lock();
        if list {
            let _ = writeln!(lock, "  Length      Date    Time    Name");
            let (mut ts, mut tc) = (0u64, 0usize);
            for e in &entries {
                if !wanted(&e.name) {
                    continue;
                }
                let _ = writeln!(
                    lock,
                    "{:>8}  ----      ----   {}",
                    e.usize_,
                    e.name.escape_ascii()
                );
                ts += e.usize_ as u64;
                tc += 1;
            }
            let _ = writeln!(lock, "--------                     -------");
            let _ = writeln!(lock, "{ts:>8}                     {tc} files");
            return Ok(0);
        }
        let base: PathBuf = dest.unwrap_or_else(|| PathBuf::from("."));
        let mut rc = 0;
        for e in &entries {
            if !wanted(&e.name) {
                continue;
            }
            let payload = match zip_entry_data(&data, e) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("unzip: skipping {}: {err}", e.name.escape_ascii());
                    rc = 1;
                    continue;
                }
            };
            if to_pipe {
                if lock.write_all(&payload).is_err() {
                    return Ok(1);
                }
                continue;
            }
            let rel = String::from_utf8_lossy(&e.name);
            if rel.ends_with('/') {
                if std::fs::create_dir_all(base.join(rel.as_ref())).is_err() {
                    eprintln!("unzip: cannot create dir {rel}");
                    rc = 1;
                }
                continue;
            }
            let target = base.join(rel.as_ref());
            if let Some(par) = target.parent() {
                if std::fs::create_dir_all(par).is_err() {
                    eprintln!("unzip: cannot create dir for {rel}");
                    rc = 1;
                    continue;
                }
            }
            if target.exists() && (never || !overwrite) {
                eprintln!("unzip: {rel} exists, skipping (use -o to overwrite)");
                continue;
            }
            if std::fs::write(&target, &payload).is_err() {
                eprintln!("unzip: cannot write {rel}");
                rc = 1;
            }
        }
        Ok(rc)
    }
}
