use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub fn parse_ar(data: &[u8]) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    if data.len() < 8 || &data[..8] != b"!<arch>\n" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad ar magic",
        ));
    }
    let mut out = Vec::new();
    let mut p = 8;
    while p + 60 <= data.len() {
        let name_raw = &data[p..p + 16];
        let mut name = String::from_utf8_lossy(name_raw).into_owned();
        if let Some(s) = name.find('/') {
            name.truncate(s);
        }
        name = name.trim().to_owned();
        let size: usize = String::from_utf8_lossy(&data[p + 48..p + 58])
            .trim()
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad ar size"))?;
        if &data[p + 58..p + 60] != b"\x60\x0a" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad ar member trailer",
            ));
        }
        p += 60;
        if p + size > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "truncated ar member",
            ));
        }
        out.push((name, data[p..p + size].to_vec()));
        p += size;
        if p % 2 != 0 {
            p += 1;
        }
    }
    Ok(out)
}

fn parse_octal(b: &[u8]) -> u64 {
    let mut v = 0u64;
    for &c in b {
        if c == 0 || c == b' ' {
            if v != 0 {
                break;
            }
            continue;
        }
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        v = v * 8 + (c - b'0') as u64;
    }
    v
}

pub struct TarEntry {
    name: String,
    size: u64,
    data_off: usize,
    dir: bool,
}

pub fn parse_ustar(data: &[u8]) -> std::io::Result<Vec<TarEntry>> {
    let mut out = Vec::new();
    let mut p = 0;
    while p + 512 <= data.len() {
        let blk = &data[p..p + 512];
        if blk.iter().all(|&b| b == 0) {
            break;
        }
        let mut name = String::from_utf8_lossy(&blk[..100])
            .trim_matches('\0')
            .to_owned();
        let prefix = String::from_utf8_lossy(&blk[345..500])
            .trim_matches('\0')
            .to_owned();
        if !prefix.is_empty() {
            name = format!("{prefix}/{name}");
        }
        let size = parse_octal(&blk[124..136]);
        let tflag = blk[156];
        let end = (p + 512 + size as usize).div_ceil(512) * 512;
        if end > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "truncated tar member",
            ));
        }
        out.push(TarEntry {
            name,
            size,
            data_off: p + 512,
            dir: tflag == b'5',
        });
        p = end;
    }
    Ok(out)
}

pub fn maybe_decompressed_tar(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if is_gzip(data) {
        return decode_gzip_members(data);
    }
    Ok(data.to_vec())
}

pub fn deb_control_field(control_tar: &[u8], field: &str) -> String {
    let Ok(entries) = parse_ustar(control_tar) else {
        return String::new();
    };
    for e in &entries {
        if e.name.ends_with("control") && !e.dir {
            let end = (e.data_off + e.size as usize).min(control_tar.len());
            let text = String::from_utf8_lossy(&control_tar[e.data_off..end]);
            for line in text.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    if k.trim().eq_ignore_ascii_case(field) {
                        return v.trim().to_owned();
                    }
                }
            }
        }
    }
    String::new()
}

pub fn deb_parts(data: &[u8]) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let members = parse_ar(data)?;
    let mut control: Option<Vec<u8>> = None;
    let mut payload: Option<Vec<u8>> = None;
    for (name, bytes) in members {
        if name.starts_with("control.tar") {
            control = Some(maybe_decompressed_tar(&bytes)?);
        } else if name.starts_with("data.tar") {
            payload = Some(maybe_decompressed_tar(&bytes)?);
        }
    }
    match (control, payload) {
        (Some(c), Some(d)) => Ok((c, d)),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "deb lacks control.tar/data.tar",
        )),
    }
}

pub fn extract_ustar(data: &[u8], dest: &Path, list_only: bool) -> std::io::Result<u32> {
    let entries = parse_ustar(data)?;
    let out = std::io::stdout();
    let mut lock = out.lock();
    let mut n = 0;
    for e in &entries {
        if list_only {
            let _ = writeln!(lock, "{} {}", e.size, e.name);
            continue;
        }
        let target = dest.join(&e.name);
        if e.dir {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(par) = target.parent() {
            std::fs::create_dir_all(par)?;
        }
        let end = (e.data_off + e.size as usize).min(data.len());
        std::fs::write(&target, &data[e.data_off..end])?;
        n += 1;
    }
    Ok(n)
}

pub struct DpkgDebApplet;
impl Applet for DpkgDebApplet {
    fn name(&self) -> &'static str {
        "dpkg-deb"
    }
    fn description(&self) -> &'static str {
        "Deb archive tool (-f field, -c contents, -e/-x extract subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let a: Vec<&[u8]> = args.iter().map(ab).collect();
        if a.is_empty() {
            eprintln!("dpkg-deb: need an action (-f/-c/-e/-x/--info/--contents/--extract)");
            return Ok(1);
        }
        let act = a[0];
        let (action, mut rest): (&str, Vec<&OsString>) = if act == b"-f" || act == b"--field" {
            ("field", args[1..].iter().collect())
        } else if act == b"-c" || act == b"--contents" || act == b"--ctrl-tarfile" {
            ("contents", args[1..].iter().collect())
        } else if act == b"-e" || act == b"--control" || act == b"--info" {
            ("control", args[1..].iter().collect())
        } else if act == b"-x" || act == b"--extract" || act == b"--fsys-tarfile" {
            ("extract", args[1..].iter().collect())
        } else {
            eprintln!("dpkg-deb: unknown action {}", act.escape_ascii());
            return Ok(1);
        };
        if rest.is_empty() {
            eprintln!("dpkg-deb: missing deb file");
            return Ok(1);
        }

        let mut fields: Vec<String> = Vec::new();
        let debpath: &OsString;
        if action == "field" && rest.len() >= 2 {
            debpath = rest[0];
            fields = rest[1..]
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
        } else {
            debpath = rest[0];
            rest = rest[1..].to_vec();
        }
        let data = match std::fs::read(Path::new(debpath)) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("dpkg-deb: {}: {e}", debpath.to_string_lossy());
                return Ok(1);
            }
        };
        let (control, payload) = match deb_parts(&data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("dpkg-deb: {e}");
                return Ok(1);
            }
        };
        match action {
            "field" => {
                let out = std::io::stdout();
                let mut lock = out.lock();
                if fields.is_empty() {
                    let entries = parse_ustar(&control).map_err(|e| crate::core::BbError::Io {
                        path: None,
                        source: e,
                    })?;
                    for e in &entries {
                        if e.name.ends_with("control") && !e.dir {
                            let end = (e.data_off + e.size as usize).min(control.len());
                            let _ = lock.write_all(&control[e.data_off..end]);
                        }
                    }
                } else {
                    for f in &fields {
                        let _ = writeln!(lock, "{}", deb_control_field(&control, f));
                    }
                }
                Ok(0)
            }
            "contents" => {
                if extract_ustar(&payload, Path::new("."), true).is_err() {
                    eprintln!("dpkg-deb: bad data.tar");
                    return Ok(1);
                }
                Ok(0)
            }
            "control" => {
                let dest = rest
                    .first()
                    .map(PathBuf::from)
                    .unwrap_or(PathBuf::from("."));
                if extract_ustar(&control, &dest, false).is_err() {
                    eprintln!("dpkg-deb: bad control.tar");
                    return Ok(1);
                }
                Ok(0)
            }
            _ => {
                let dest = rest
                    .first()
                    .map(PathBuf::from)
                    .unwrap_or(PathBuf::from("."));
                if extract_ustar(&payload, &dest, false).is_err() {
                    eprintln!("dpkg-deb: bad data.tar");
                    return Ok(1);
                }
                Ok(0)
            }
        }
    }
}
