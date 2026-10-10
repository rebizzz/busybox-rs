use crate::applets::archival::inflate::{inflate_all, inflate_all_consumed};
use crate::core::{Applet, Result};
use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ab(a: &OsString) -> &[u8] {
    a.as_bytes()
}

fn u16le(d: &[u8], o: usize) -> usize {
    d[o] as usize | ((d[o + 1] as usize) << 8)
}

fn u32le(d: &[u8], o: usize) -> u32 {
    d[o] as u32 | ((d[o + 1] as u32) << 8) | ((d[o + 2] as u32) << 16) | ((d[o + 3] as u32) << 24)
}

fn u32be(d: &[u8], o: usize) -> u32 {
    ((d[o] as u32) << 24) | ((d[o + 1] as u32) << 16) | ((d[o + 2] as u32) << 8) | (d[o + 3] as u32)
}

fn read_all_bytes(p: Option<&Path>) -> std::io::Result<Vec<u8>> {
    let mut v = Vec::new();
    match p {
        Some(path) => File::open(path)?.read_to_end(&mut v)?,
        None => std::io::stdin().read_to_end(&mut v)?,
    };
    Ok(v)
}

fn stdout_write(data: &[u8]) -> std::io::Result<()> {
    std::io::stdout().lock().write_all(data)
}

fn copy_stream<R: Read, W: Write>(r: &mut R, w: &mut W) -> std::io::Result<u64> {
    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        w.write_all(&buf[..n])?;
        total += n as u64;
    }
    Ok(total)
}

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut tab = [0u32; 256];
    for (i, slot) in tab.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = tab[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn decode_gzip_members(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut any = false;
    while pos < data.len() {
        if data.len() - pos < 10 || data[pos] != 0x1f || data[pos + 1] != 0x8b || data[pos + 2] != 8
        {
            if !any {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "not in gzip format",
                ));
            }
            break;
        }
        let flg = data[pos + 3];
        let mut p = pos + 10;
        if flg & 0x04 != 0 {
            if p + 2 > data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "bad gzip extra",
                ));
            }
            let xlen = u16le(data, p);
            p += 2 + xlen;
        }
        if flg & 0x08 != 0 {
            while p < data.len() && data[p] != 0 {
                p += 1;
            }
            p += 1;
        }
        if flg & 0x10 != 0 {
            while p < data.len() && data[p] != 0 {
                p += 1;
            }
            p += 1;
        }
        if flg & 0x02 != 0 {
            p += 2;
        }
        if p > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "bad gzip header",
            ));
        }
        let (chunk, used) = inflate_all_consumed(&data[p..])?;
        p += used;
        if p + 8 > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "missing gzip trailer",
            ));
        }
        let want_crc = u32le(data, p);
        let want_size = u32le(data, p + 4);
        if want_crc != crc32_ieee(&chunk) || want_size != chunk.len() as u32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "gzip CRC/size mismatch",
            ));
        }
        out.extend_from_slice(&chunk);
        pos = p + 8;
        any = true;
    }
    Ok(out)
}

fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

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

pub struct ZcatApplet;
impl Applet for ZcatApplet {
    fn name(&self) -> &'static str {
        "zcat"
    }
    fn description(&self) -> &'static str {
        "Decompress gzip members to stdout; non-gzip input is copied through"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&OsString> = args
            .iter()
            .filter(|a| ab(a) != b"-f" && ab(a) != b"--force")
            .collect();
        let mut rc = 0;
        let out = std::io::stdout();
        let mut lock = out.lock();
        if files.is_empty() {
            let mut data = Vec::new();
            if std::io::stdin().read_to_end(&mut data).is_err() {
                return Ok(1);
            }
            let payload = if is_gzip(&data) {
                match decode_gzip_members(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("zcat: {e}");
                        return Ok(1);
                    }
                }
            } else {
                data
            };
            if lock.write_all(&payload).is_err() {
                return Ok(1);
            }
            return Ok(0);
        }
        for f in files {
            let data = match std::fs::read(Path::new(f)) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("zcat: {}: {e}", f.to_string_lossy());
                    rc = 1;
                    continue;
                }
            };

            let payload = if is_gzip(&data) {
                match decode_gzip_members(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("zcat: {}: {e}", f.to_string_lossy());
                        rc = 1;
                        continue;
                    }
                }
            } else {
                data
            };
            if lock.write_all(&payload).is_err() {
                return Ok(1);
            }
        }
        Ok(rc)
    }
}

struct RpmHeader {
    tags: Vec<(u32, u32, Vec<u8>, u32)>,
}

fn parse_rpm_header(data: &[u8], off: usize) -> std::io::Result<(RpmHeader, usize)> {
    if off + 16 > data.len() || &data[off..off + 3] != b"\x8e\xad\xe8" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad rpm header magic",
        ));
    }
    let count = u32be(data, off + 8) as usize;
    let size = u32be(data, off + 12) as usize;
    let mut p = off + 16;
    let store = off + 16 + count * 16;
    let mut tags = Vec::new();
    for _ in 0..count {
        if p + 16 > data.len() || store + size > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "truncated rpm header",
            ));
        }
        let tag = u32be(data, p);
        let typ = u32be(data, p + 4);
        let ofs = u32be(data, p + 8) as usize;
        let cnt = u32be(data, p + 12);
        let start = store + ofs;

        let el = match typ {
            1 | 2 | 7 => 1,
            3 => 2,
            4 => 4,
            5 => 8,
            6 | 8 | 9 => 0,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bad rpm tag type",
                ));
            }
        };
        let raw = if el == 0 {
            let mut end = start;
            for _ in 0..cnt {
                while end < store + size && data[end] != 0 {
                    end += 1;
                }
                end += 1;
            }
            if end > store + size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bad rpm string",
                ));
            }
            data[start..end.min(store + size)].to_vec()
        } else {
            let n = ofs + el * cnt as usize;
            if n > size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "bad rpm tag range",
                ));
            }
            data[start..start + el * cnt as usize].to_vec()
        };
        tags.push((tag, typ, raw, cnt));
        p += 16;
    }
    Ok((RpmHeader { tags }, store + size))
}

impl RpmHeader {
    fn str_tag(&self, tag: u32) -> String {
        for (t, _, raw, _) in &self.tags {
            if *t == tag {
                let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                return String::from_utf8_lossy(&raw[..end]).into_owned();
            }
        }
        String::new()
    }
}

struct RpmPkg {
    hdr: RpmHeader,
    payload: Vec<u8>,
    payload_comp: String,
}

fn parse_rpm(data: &[u8]) -> std::io::Result<RpmPkg> {
    if data.len() < 96 || &data[0..4] != b"\xed\xab\xee\xdb" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad rpm lead magic",
        ));
    }
    let mut off = 96;

    let (_, next) = parse_rpm_header(data, off)?;
    off = next;

    while off % 8 != 0 {
        off += 1;
    }
    let (hdr, pend) = parse_rpm_header(data, off)?;
    let comp = hdr.str_tag(1125);
    Ok(RpmPkg {
        hdr,
        payload: data[pend..].to_vec(),
        payload_comp: comp,
    })
}

fn rpm_payload_cpio(pkg: &RpmPkg) -> std::io::Result<Vec<u8>> {
    if pkg.payload_comp.contains("gzip") || is_gzip(&pkg.payload) {
        return decode_gzip_members(&pkg.payload);
    }
    if pkg.payload_comp.is_empty() || pkg.payload_comp.contains("uncompressed") {
        return Ok(pkg.payload.clone());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("unsupported rpm payload compressor '{}'", pkg.payload_comp),
    ))
}

pub struct RpmApplet;
impl Applet for RpmApplet {
    fn name(&self) -> &'static str {
        "rpm"
    }
    fn description(&self) -> &'static str {
        "Query rpm headers (-qpi subset) and dump cpio payload"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let a: Vec<&[u8]> = args.iter().map(ab).collect();
        let query = a
            .iter()
            .any(|x| x.len() > 1 && x[0] == b'-' && x.contains(&b'q'));
        if a.is_empty() || !query {
            eprintln!("rpm: only -q supported (try: rpm -qpi pkg.rpm)");
            return Ok(1);
        }
        let info = a.iter().any(|x| x.contains(&b'i'));
        let mut rc = 0;
        for f in args.iter().filter(|x| {
            let b = ab(x);
            !b.starts_with(b"-")
        }) {
            let data = match std::fs::read(Path::new(f)) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("rpm: {}: {e}", f.to_string_lossy());
                    rc = 1;
                    continue;
                }
            };
            match parse_rpm(&data) {
                Ok(pkg) => {
                    let h = &pkg.hdr;
                    println!(
                        "{}-{}-{} {}",
                        h.str_tag(1000),
                        h.str_tag(1001),
                        h.str_tag(1002),
                        h.str_tag(1022)
                    );
                    if info {
                        println!("Summary : {}", h.str_tag(1004));
                        println!("Packager: {}", h.str_tag(1015));
                    }
                }
                Err(e) => {
                    eprintln!("rpm: {}: {e}", f.to_string_lossy());
                    rc = 1;
                }
            }
        }
        Ok(rc)
    }
}

pub struct Rpm2cpioApplet;
impl Applet for Rpm2cpioApplet {
    fn name(&self) -> &'static str {
        "rpm2cpio"
    }
    fn description(&self) -> &'static str {
        "Write rpm payload as (decompressed) cpio archive to stdout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("rpm2cpio: missing rpm file");
            return Ok(1);
        }
        let data = match std::fs::read(Path::new(&args[0])) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("rpm2cpio: {}: {e}", args[0].to_string_lossy());
                return Ok(1);
            }
        };
        let pkg = match parse_rpm(&data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("rpm2cpio: {e}");
                return Ok(1);
            }
        };
        match rpm_payload_cpio(&pkg) {
            Ok(cpio) => {
                if stdout_write(&cpio).is_err() {
                    return Ok(1);
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("rpm2cpio: {e}");
                Ok(1)
            }
        }
    }
}

fn parse_ar(data: &[u8]) -> std::io::Result<Vec<(String, Vec<u8>)>> {
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

struct TarEntry {
    name: String,
    size: u64,
    data_off: usize,
    dir: bool,
}

fn parse_ustar(data: &[u8]) -> std::io::Result<Vec<TarEntry>> {
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

fn maybe_decompressed_tar(data: &[u8]) -> std::io::Result<Vec<u8>> {
    if is_gzip(data) {
        return decode_gzip_members(data);
    }
    Ok(data.to_vec())
}

fn deb_control_field(control_tar: &[u8], field: &str) -> String {
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

fn deb_parts(data: &[u8]) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
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

fn extract_ustar(data: &[u8], dest: &Path, list_only: bool) -> std::io::Result<u32> {
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

fn dpkg_status_db() -> PathBuf {
    std::env::var_os("DPKG_ADMINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/var/lib/dpkg"))
        .join("status")
}

pub struct DpkgApplet;
impl Applet for DpkgApplet {
    fn name(&self) -> &'static str {
        "dpkg"
    }
    fn description(&self) -> &'static str {
        "Deb package info/install subset (-l/-s list, -i/--unpack install)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let a: Vec<&[u8]> = args.iter().map(ab).collect();
        if a.is_empty() {
            eprintln!("dpkg: need -l/-s/-L/-i/--unpack");
            return Ok(1);
        }
        let act = a[0];
        if act == b"-l" || act == b"--list" {
            let pat = if a.len() > 1 {
                Some(String::from_utf8_lossy(a[1]).into_owned())
            } else {
                None
            };
            let text = std::fs::read_to_string(dpkg_status_db()).unwrap_or_default();
            let out = std::io::stdout();
            let mut lock = out.lock();
            let _ = writeln!(lock, "Desired=Unknown/Install/Remove/Purge/Hold");
            let mut pkg = String::new();
            let mut status = String::new();
            let mut ver = String::new();
            let mut desc = String::new();
            let flush = |pkg: &str, status: &str, ver: &str, desc: &str, lock: &mut dyn Write| {
                let ok = status.contains("installed");
                let _ = writeln!(
                    lock,
                    "{}  {pkg:<30} {ver:<16} {desc}",
                    if ok { "ii" } else { "un" }
                );
            };
            for line in text.lines().chain([""]) {
                if line.is_empty() {
                    if !pkg.is_empty() {
                        let hit = pat.as_ref().is_none_or(|p| pkg.contains(p));
                        if hit {
                            flush(&pkg, &status, &ver, &desc, &mut lock);
                        }
                    }
                    pkg.clear();
                    status.clear();
                    ver.clear();
                    desc.clear();
                } else if let Some((k, v)) = line.split_once(':') {
                    match k {
                        "Package" => pkg = v.trim().to_owned(),
                        "Status" => status = v.trim().to_owned(),
                        "Version" => ver = v.trim().to_owned(),
                        "Description" => desc = v.trim().to_owned(),
                        _ => {}
                    }
                }
            }
            return Ok(0);
        }
        if act == b"-s" || act == b"--status" {
            if a.len() < 2 {
                eprintln!("dpkg: -s needs a package name");
                return Ok(1);
            }
            let want = String::from_utf8_lossy(a[1]).into_owned();
            let text = match std::fs::read_to_string(dpkg_status_db()) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("dpkg: status database unavailable");
                    return Ok(1);
                }
            };
            let mut block = String::new();
            let mut found = false;
            for line in text.lines().chain([""]) {
                if line.is_empty() {
                    if block.lines().any(|l| l == format!("Package: {want}")) {
                        println!("{block}");
                        found = true;
                    }
                    block.clear();
                } else {
                    block.push_str(line);
                    block.push('\n');
                }
            }
            if !found {
                eprintln!("dpkg: package '{want}' not installed");
                return Ok(1);
            }
            return Ok(0);
        }
        if act == b"-i" || act == b"--install" || act == b"--unpack" {
            if a.len() < 2 {
                eprintln!("dpkg: --install needs a deb file");
                return Ok(1);
            }
            let mut rc = 0;
            for f in &args[1..] {
                let data = match std::fs::read(Path::new(f)) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("dpkg: {}: {e}", f.to_string_lossy());
                        rc = 1;
                        continue;
                    }
                };
                match deb_parts(&data) {
                    Ok((_, payload)) => {
                        let root = std::env::var_os("DPKG_ROOT")
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from("/"));
                        if extract_ustar(&payload, &root, false).is_err() {
                            eprintln!("dpkg: failed to unpack {}", f.to_string_lossy());
                            rc = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("dpkg: {}: {e}", f.to_string_lossy());
                        rc = 1;
                    }
                }
            }
            return Ok(rc);
        }
        eprintln!("dpkg: unsupported action {}", act.escape_ascii());
        Ok(1)
    }
}

const fn ior(magic: u8, nr: u8, size: usize) -> u64 {
    0x8000_0000 | ((size as u64) << 16) | ((magic as u64) << 8) | nr as u64
}

const fn iow(magic: u8, nr: u8, size: usize) -> u64 {
    0x4000_0000 | ((size as u64) << 16) | ((magic as u64) << 8) | nr as u64
}

#[repr(C)]
struct MtdInfoUser {
    typ: u8,
    _pad: [u8; 3],
    flags: u32,
    size: u32,
    erasesize: u32,
    writesize: u32,
    oobsize: u32,
    _pad2: u64,
}

#[repr(C)]
struct EraseInfoUser {
    start: u32,
    length: u32,
}

#[repr(C)]
struct Mtop {
    op: i16,
    count: i32,
}

#[repr(C)]
struct UbiAttachReq {
    ubi_num: i32,
    mtd_num: i32,
    vid_hdr_offset: i32,
    padding: [u8; 12],
}

fn ioctl_err(dev: &str, e: std::io::Error) -> i32 {
    eprintln!("{dev}: ioctl: {e} (no such device or no hardware?)");
    1
}

fn open_dev(path: &str) -> std::io::Result<File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
}

pub struct NanddumpApplet;
impl Applet for NanddumpApplet {
    fn name(&self) -> &'static str {
        "nanddump"
    }
    fn description(&self) -> &'static str {
        "Dump NAND flash (MEMGETINFO ioctl; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/mtd0");
        let mut file: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("nanddump: -f requires an argument");
                    return Ok(1);
                }
                file = Some(PathBuf::from(&args[i]));
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("nanddump: {dev}: {e}");
                return Ok(1);
            }
        };
        let mut info = MtdInfoUser {
            typ: 0,
            _pad: [0; 3],
            flags: 0,
            size: 0,
            erasesize: 0,
            writesize: 0,
            oobsize: 0,
            _pad2: 0,
        };
        let req = ior(b'M', 1, size_of::<MtdInfoUser>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), req, &mut info) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        eprintln!(
            "nanddump: {dev}: size={} erasesize={}",
            info.size, info.erasesize
        );
        let mut src = &f;
        let out = std::io::stdout();
        let mut lock = out.lock();
        let mut total = 0u64;
        if let Some(fp) = file {
            let mut dst = match File::create(&fp) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("nanddump: {}: {e}", fp.display());
                    return Ok(1);
                }
            };
            match copy_stream(&mut src, &mut dst) {
                Ok(n) => total = n,
                Err(e) => {
                    eprintln!("nanddump: {e}");
                    return Ok(1);
                }
            }
        } else if copy_stream(&mut src, &mut lock).is_err() {
            return Ok(1);
        }
        eprintln!("nanddump: dumped {total} bytes");
        Ok(0)
    }
}

pub struct NandwriteApplet;
impl Applet for NandwriteApplet {
    fn name(&self) -> &'static str {
        "nandwrite"
    }
    fn description(&self) -> &'static str {
        "Write to NAND flash (MEMERASE+write; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<String> = None;
        let mut input: Option<PathBuf> = None;
        let mut start: u32 = 0;
        for a in args {
            let b = ab(a);

            if b.starts_with(b"-") {
                continue;
            }
            if dev.is_none() {
                dev = Some(String::from_utf8_lossy(b).into_owned());
            } else if input.is_none() {
                input = Some(PathBuf::from(a));
            }
        }

        let mut si = 0;
        while si < args.len() {
            if ab(&args[si]) == b"-s" && si + 1 < args.len() {
                start = String::from_utf8_lossy(ab(&args[si + 1]))
                    .parse()
                    .unwrap_or(0);
            }
            si += 1;
        }
        let dev = match dev {
            Some(d) => d,
            None => {
                eprintln!("nandwrite: missing mtd device");
                return Ok(1);
            }
        };
        let data = match read_all_bytes(input.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("nandwrite: input: {e}");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("nandwrite: {dev}: {e}");
                return Ok(1);
            }
        };
        let erase = EraseInfoUser {
            start,
            length: data.len() as u32,
        };
        let req = iow(b'M', 2, size_of::<EraseInfoUser>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), req, &erase) };
        if r != 0 {
            eprintln!(
                "nandwrite: erase skipped: {}",
                std::io::Error::last_os_error()
            );
        }
        use std::os::unix::fs::FileExt;
        match f.write_at(&data, u64::from(start)) {
            Ok(n) => {
                eprintln!("nandwrite: wrote {n} bytes to {dev}");
                Ok(0)
            }
            Err(e) => {
                eprintln!("nandwrite: {dev}: {e}");
                Ok(1)
            }
        }
    }
}

fn ubi_ctrl_open(given: Option<&str>) -> std::io::Result<(File, String)> {
    let dev = given.unwrap_or("/dev/ubi_ctrl").to_owned();
    open_dev(&dev).map(|f| (f, dev))
}

pub struct UbiattachApplet;
impl Applet for UbiattachApplet {
    fn name(&self) -> &'static str {
        "ubiattach"
    }
    fn description(&self) -> &'static str {
        "Attach MTD to UBI (UBI_CTRL_IOCATT; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mtd: Option<i32> = None;
        let mut ubi: i32 = -1;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"-m" || b == b"--mtdn") && i + 1 < args.len() {
                mtd = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 1;
            } else if (b == b"-d" || b == b"--devn") && i + 1 < args.len() {
                ubi = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(-1);
                i += 1;
            }
            i += 1;
        }
        let mtd = match mtd {
            Some(m) => m,
            None => {
                eprintln!("ubiattach: missing -m MTD number");
                return Ok(1);
            }
        };
        let (f, dev) = match ubi_ctrl_open(None) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("ubiattach: /dev/ubi_ctrl: {e}");
                return Ok(1);
            }
        };
        let req_data = UbiAttachReq {
            ubi_num: ubi,
            mtd_num: mtd,
            vid_hdr_offset: 0,
            padding: [0; 12],
        };

        let code = iow(b'O', 64, size_of::<UbiAttachReq>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), code, &req_data) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}

pub struct UbidetachApplet;
impl Applet for UbidetachApplet {
    fn name(&self) -> &'static str {
        "ubidetach"
    }
    fn description(&self) -> &'static str {
        "Detach UBI device (UBI_CTRL_IOCDET; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ubi: Option<i32> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"-d" || b == b"--devn") && i + 1 < args.len() {
                ubi = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 1;
            }
            i += 1;
        }
        let ubi = match ubi {
            Some(u) => u,
            None => {
                eprintln!("ubidetach: missing -d UBI number");
                return Ok(1);
            }
        };
        let (f, dev) = match ubi_ctrl_open(None) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("ubidetach: /dev/ubi_ctrl: {e}");
                return Ok(1);
            }
        };
        let code = iow(b'O', 65, size_of::<i32>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), code, &ubi) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}

fn ubi_vol_ioctl(dev: &str, nr: u8, payload: &[u8]) -> i32 {
    let f = match open_dev(dev) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{dev}: {e}");
            return 1;
        }
    };

    let code = iow(b'o', nr, payload.len());
    let r = unsafe { libc::ioctl(f.as_raw_fd(), code, payload.as_ptr()) };
    if r != 0 {
        return ioctl_err(dev, std::io::Error::last_os_error());
    }
    0
}

pub struct UbimkvolApplet;
impl Applet for UbimkvolApplet {
    fn name(&self) -> &'static str {
        "ubimkvol"
    }
    fn description(&self) -> &'static str {
        "Create UBI volume (UBI_IOCMKVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut name = String::new();
        let mut size: i64 = 0;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-N" && i + 1 < args.len() {
                name = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 1;
            } else if b == b"-s" && i + 1 < args.len() {
                size = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                i += 1;
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        if name.is_empty() {
            eprintln!("ubimkvol: missing -N name");
            return Ok(1);
        }

        let mut req = vec![0u8; 152];
        req[0..4].copy_from_slice(&(-1i32).to_le_bytes());
        req[4..8].copy_from_slice(&1i32.to_le_bytes());
        req[8..16].copy_from_slice(&size.to_le_bytes());
        req[16] = 3;
        let nb = name.as_bytes();
        let nl = nb.len().min(127) as i16;
        req[20..22].copy_from_slice(&nl.to_le_bytes());
        req[24..24 + nl as usize].copy_from_slice(&nb[..nl as usize]);
        Ok(ubi_vol_ioctl(&dev, 0, &req))
    }
}

pub struct UbirenameApplet;
impl Applet for UbirenameApplet {
    fn name(&self) -> &'static str {
        "ubirename"
    }
    fn description(&self) -> &'static str {
        "Rename UBI volume (UBI_IOCRNVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut pair: Option<(String, String)> = None;
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") {
                continue;
            }
            let s = String::from_utf8_lossy(b).into_owned();
            if s.contains('=') && pair.is_none() {
                let (o, n) = s.split_once('=').unwrap_or(("", ""));
                pair = Some((o.to_owned(), n.to_owned()));
            } else {
                dev = s;
            }
        }

        let positional: Vec<String> = args
            .iter()
            .filter(|a| !ab(a).starts_with(b"-"))
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        if pair.is_none() && positional.len() >= 3 {
            dev = positional[0].clone();
            pair = Some((positional[1].clone(), positional[2].clone()));
        }
        let (old, new) = match pair {
            Some(p) => p,
            None => {
                eprintln!("ubirename: usage: ubirename DEV old=new");
                return Ok(1);
            }
        };

        let ob = old.as_bytes();
        let nbv = new.as_bytes();
        let mut req = vec![0u8; 16 + 140 * 2];
        req[0..4].copy_from_slice(&2i32.to_le_bytes());
        req[16..18].copy_from_slice(&(ob.len().min(127) as i16).to_le_bytes());
        req[20..20 + ob.len().min(127)].copy_from_slice(&ob[..ob.len().min(127)]);
        req[156..158].copy_from_slice(&(nbv.len().min(127) as i16).to_le_bytes());
        req[160..160 + nbv.len().min(127)].copy_from_slice(&nbv[..nbv.len().min(127)]);
        Ok(ubi_vol_ioctl(&dev, 1, &req))
    }
}

pub struct UbirmvolApplet;
impl Applet for UbirmvolApplet {
    fn name(&self) -> &'static str {
        "ubirmvol"
    }
    fn description(&self) -> &'static str {
        "Remove UBI volume (UBI_IOCRMVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut name = String::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-N" && i + 1 < args.len() {
                name = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 1;
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        if name.is_empty() {
            eprintln!("ubirmvol: missing -N name");
            return Ok(1);
        }
        let mut req = vec![0u8; 136];
        let nb = name.as_bytes();
        req[0..2].copy_from_slice(&(nb.len().min(127) as i16).to_le_bytes());
        req[8..8 + nb.len().min(127)].copy_from_slice(&nb[..nb.len().min(127)]);
        Ok(ubi_vol_ioctl(&dev, 2, &req))
    }
}

pub struct UbirsvolApplet;
impl Applet for UbirsvolApplet {
    fn name(&self) -> &'static str {
        "ubirsvol"
    }
    fn description(&self) -> &'static str {
        "Resize UBI volume (UBI_IOCRSVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0_0");
        let mut size: i64 = 0;
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") {
                continue;
            }
            let s = String::from_utf8_lossy(b).into_owned();
            if let Ok(n) = s.parse::<i64>() {
                size = n;
            } else {
                dev = s;
            }
        }

        let mut req = vec![0u8; 16];
        req[8..16].copy_from_slice(&size.to_le_bytes());
        Ok(ubi_vol_ioctl(&dev, 3, &req))
    }
}

pub struct UbiupdatevolApplet;
impl Applet for UbiupdatevolApplet {
    fn name(&self) -> &'static str {
        "ubiupdatevol"
    }
    fn description(&self) -> &'static str {
        "Update UBI volume from stdin/file (UBI_IOCVOLUP; graceful error w/o hw)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<String> = None;
        let mut input: Option<PathBuf> = None;
        for a in args {
            let b = ab(a);
            if b == b"-t" || b.starts_with(b"-") {
                continue;
            }
            if dev.is_none() {
                dev = Some(String::from_utf8_lossy(b).into_owned());
            } else {
                input = Some(PathBuf::from(a));
            }
        }
        let dev = dev.unwrap_or_else(|| String::from("/dev/ubi0_0"));
        let data = match read_all_bytes(input.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("ubiupdatevol: input: {e}");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("ubiupdatevol: {dev}: {e}");
                return Ok(1);
            }
        };

        let mut req = [0u8; 16];
        req[8..16].copy_from_slice(&(data.len() as i64).to_le_bytes());
        let code = iow(b'o', 0, 16);
        if unsafe { libc::ioctl(f.as_raw_fd(), code, req.as_ptr()) } != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        use std::os::unix::fs::FileExt;
        let mut off = 0u64;
        let mut left = data.as_slice();
        while !left.is_empty() {
            match f.write_at(left, off) {
                Ok(0) => break,
                Ok(n) => {
                    off += n as u64;
                    left = &left[n..];
                }
                Err(e) => {
                    eprintln!("ubiupdatevol: {dev}: {e}");
                    return Ok(1);
                }
            }
        }
        Ok(0)
    }
}

pub struct MtApplet;
impl Applet for MtApplet {
    fn name(&self) -> &'static str {
        "mt"
    }
    fn description(&self) -> &'static str {
        "Tape drive control (MTIOCTOP ioctl; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = std::env::var_os("TAPE")
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("/dev/tape"));
        let mut op: Option<i16> = None;
        let mut count: i32 = 1;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-f" && i + 1 < args.len() {
                dev = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 2;
                continue;
            }
            let s = String::from_utf8_lossy(b).to_lowercase();
            let code: Option<i16> = match s.as_str() {
                "eof" | "weof" => Some(5),
                "fsf" => Some(1),
                "bsf" => Some(2),
                "fsr" => Some(3),
                "bsr" => Some(4),
                "rewind" | "rew" => Some(6),
                "offline" | "eject" | "rewoffl" => Some(7),
                "retension" => Some(9),
                "erase" => Some(10),
                "eom" | "seod" => Some(11),
                "status" => Some(-1),
                _ => None,
            };
            if let Some(c) = code {
                op = Some(c);
            } else if let Ok(n) = s.parse::<i32>() {
                count = n;
            }
            i += 1;
        }
        let op = match op {
            Some(o) => o,
            None => {
                eprintln!("mt: missing operation (eof fsf bsf rewind offline status ...)");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("mt: {dev}: {e}");
                return Ok(1);
            }
        };
        if op == -1 {
            let mut buf = [0u8; 48];
            let code = ior(b'm', 2, buf.len());
            if unsafe { libc::ioctl(f.as_raw_fd(), code, buf.as_mut_ptr()) } != 0 {
                return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
            }
            println!("mt: status ok ({dev})");
            return Ok(0);
        }
        let m = Mtop { op, count };
        let code = iow(b'm', 1, size_of::<Mtop>());
        if unsafe { libc::ioctl(f.as_raw_fd(), code, &m) } != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}

fn rx_crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

pub struct RxApplet;
impl Applet for RxApplet {
    fn name(&self) -> &'static str {
        "rx"
    }
    fn description(&self) -> &'static str {
        "Receive file via XMODEM (128/1K blocks, CRC or checksum)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dest: Option<PathBuf> = None;
        for a in args {
            if !ab(a).starts_with(b"-") {
                dest = Some(PathBuf::from(a));
            }
        }
        let dest = match dest {
            Some(d) => d,
            None => {
                eprintln!("rx: missing destination file");
                return Ok(1);
            }
        };
        let mut out = match File::create(&dest) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("rx: {}: {e}", dest.display());
                return Ok(1);
            }
        };
        let stdin = std::io::stdin();
        let mut inp = stdin.lock();
        let stdout = std::io::stdout();
        let mut ctl = stdout.lock();

        let mut use_crc = true;
        let _ = ctl.write_all(b"C");
        let _ = ctl.flush();
        let mut buf = [0u8; 1030];
        let mut expected: u8 = 1;
        let mut retries = 0u32;
        loop {
            if retries > 25 {
                eprintln!("rx: too many errors, aborting");
                let _ = ctl.write_all(&[0x18]);
                return Ok(1);
            }
            let mut first = [0u8; 1];
            let n = match inp.read(&mut first) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("rx: {e}");
                    return Ok(1);
                }
            };
            if n == 0 {
                retries += 1;
                let _ = ctl.write_all(&[0x15]);
                continue;
            }
            match first[0] {
                0x04 => {
                    let _ = ctl.write_all(&[0x06]);
                    break;
                }
                0x18 => {
                    eprintln!("rx: cancelled by sender");
                    return Ok(1);
                }
                0x01 | 0x02 => {
                    let blklen = if first[0] == 0x01 { 128 } else { 1024 };
                    let need = blklen + 2 + if use_crc { 2 } else { 1 };
                    let mut got = 0;
                    while got < need {
                        match inp.read(&mut buf[got..need]) {
                            Ok(0) => break,
                            Ok(n) => got += n,
                            Err(e) => {
                                eprintln!("rx: {e}");
                                return Ok(1);
                            }
                        }
                    }
                    if got < need {
                        retries += 1;
                        let _ = ctl.write_all(&[0x15]);
                        continue;
                    }
                    let seq = buf[0];
                    let seqc = buf[1];
                    if seq != expected || seqc != expected ^ 0xFF {
                        if seq == expected.wrapping_sub(1) {
                            let _ = ctl.write_all(&[0x06]);
                        } else {
                            retries += 1;
                            let _ = ctl.write_all(&[0x15]);
                        }
                        continue;
                    }
                    let payload = &buf[2..2 + blklen];
                    let ok = if use_crc {
                        let want = ((buf[2 + blklen] as u16) << 8) | buf[2 + blklen + 1] as u16;
                        rx_crc16(payload) == want
                    } else {
                        let want = buf[2 + blklen];
                        payload.iter().fold(0u8, |a, &b| a.wrapping_add(b)) == want
                    };
                    if !ok {
                        use_crc = false;
                        retries += 1;
                        let _ = ctl.write_all(&[0x15]);
                        continue;
                    }
                    if out.write_all(payload).is_err() {
                        eprintln!("rx: write error");
                        return Ok(1);
                    }
                    expected = expected.wrapping_add(1);
                    retries = 0;
                    let _ = ctl.write_all(&[0x06]);
                }
                _ => {
                    if retries == 2 {
                        use_crc = false;
                    }
                    let _ = ctl.write_all(&[0x15]);
                    retries += 1;
                }
            }
            let _ = ctl.flush();
        }
        Ok(0)
    }
}

pub struct PipeProgressApplet;
impl Applet for PipeProgressApplet {
    fn name(&self) -> &'static str {
        "pipe_progress"
    }
    fn description(&self) -> &'static str {
        "Copy stdin to stdout showing byte progress on stderr"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let stdin = std::io::stdin();
        let mut inp = stdin.lock();
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        let mut total = 0u64;
        loop {
            let n = match inp.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("pipe_progress: {e}");
                    return Ok(1);
                }
            };
            if out.write_all(&buf[..n]).is_err() {
                return Ok(1);
            }
            total += n as u64;
            eprintln!("pipe_progress: {total} bytes");
        }
        Ok(0)
    }
}

pub struct ScriptApplet;
impl Applet for ScriptApplet {
    fn name(&self) -> &'static str {
        "script"
    }
    fn description(&self) -> &'static str {
        "Run command logging session to typescript file (pty-less, timestamped)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut append = false;
        let mut file: Option<PathBuf> = None;
        let mut cmd: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-a" || b == b"--append" {
                append = true;
            } else if b == b"-c" && i + 1 < args.len() {
                cmd.push(args[i + 1].clone());
                i += 1;
            } else if !b.starts_with(b"-") {
                if file.is_none() {
                    file = Some(PathBuf::from(&args[i]));
                } else {
                    cmd.push(args[i].clone());
                }
            }
            i += 1;
        }
        let tspath = file.unwrap_or_else(|| PathBuf::from("typescript"));
        let shell_cmd = if cmd.is_empty() {
            None
        } else {
            Some(
                cmd.iter()
                    .map(|c| c.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        };
        let mut ts = match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(&tspath)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("script: {}: {e}", tspath.display());
                return Ok(1);
            }
        };
        let start = std::time::SystemTime::now();
        let _ = writeln!(ts, "Script started at {start:?}");
        let child = if let Some(sh) = shell_cmd {
            Command::new("/bin/sh").arg("-c").arg(sh).spawn()
        } else {
            Command::new("/bin/sh").spawn()
        };
        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!("script: cannot run shell: {e}");
                return Ok(1);
            }
        };

        match child.wait() {
            Ok(st) => {
                let code = st.code().unwrap_or(0);
                let _ = writeln!(
                    ts,
                    "Script done at {:?}, exit={code}",
                    std::time::SystemTime::now()
                );
                Ok(code)
            }
            Err(e) => {
                eprintln!("script: wait: {e}");
                Ok(1)
            }
        }
    }
}

pub struct ScriptreplayApplet;
impl Applet for ScriptreplayApplet {
    fn name(&self) -> &'static str {
        "scriptreplay"
    }
    fn description(&self) -> &'static str {
        "Replay typescript using timing file (scriptreplay --timing f typescript)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut timing: Option<PathBuf> = None;
        let mut tspath: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"--timing" || b == b"-t") && i + 1 < args.len() {
                timing = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            } else if !b.starts_with(b"-") {
                tspath = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }
        let (timing, tspath) = match (timing, tspath) {
            (Some(t), Some(s)) => (t, s),
            _ => {
                eprintln!("scriptreplay: usage: scriptreplay --timing timingfile typescript");
                return Ok(1);
            }
        };
        let tdata = match std::fs::read_to_string(&timing) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("scriptreplay: {}: {e}", timing.display());
                return Ok(1);
            }
        };
        let tsdata = match std::fs::read(&tspath) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("scriptreplay: {}: {e}", tspath.display());
                return Ok(1);
            }
        };
        let out = std::io::stdout();
        let mut lock = out.lock();
        let mut off = 0usize;
        for line in tdata.lines() {
            let mut it = line.split_whitespace();
            let (Some(delay), Some(size)) = (it.next(), it.next()) else {
                continue;
            };
            let delay: f64 = delay.parse().unwrap_or(0.0);
            let size: usize = size.parse().unwrap_or(0);
            if delay > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(delay.min(5.0)));
            }
            let end = (off + size).min(tsdata.len());
            if lock.write_all(&tsdata[off..end]).is_err() {
                return Ok(1);
            }
            let _ = lock.flush();
            off = end;
        }
        if off < tsdata.len() && lock.write_all(&tsdata[off..]).is_err() {
            return Ok(1);
        }
        Ok(0)
    }
}

struct OptSpec {
    short: Vec<(u8, u8)>,
    long: Vec<(String, u8)>,
}

fn short_argmode(short: &[(u8, u8)], c: u8) -> Option<u8> {
    short.iter().find(|(k, _)| *k == c).map(|(_, m)| *m)
}

fn parse_short_spec(s: &[u8]) -> Vec<(u8, u8)> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == b':' {
            i += 1;
            continue;
        }
        let c = s[i];
        let mut mode = 0;
        if i + 1 < s.len() && s[i + 1] == b':' {
            mode = 1;
            if i + 2 < s.len() && s[i + 2] == b':' {
                mode = 2;
                i += 1;
            }
            i += 1;
        }
        v.push((c, mode));
        i += 1;
    }
    v
}

fn parse_long_spec(s: &[u8]) -> Vec<(String, u8)> {
    let mut v = Vec::new();
    for part in String::from_utf8_lossy(s).split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, mode) = if let Some(n) = part.strip_suffix("::") {
            (n, 2)
        } else if let Some(n) = part.strip_suffix(':') {
            (n, 1)
        } else {
            (part, 0)
        };
        v.push((name.to_owned(), mode));
    }
    v
}

fn sh_quote(s: &[u8], out: &mut String) {
    out.push('\'');
    for &c in s {
        if c == b'\'' {
            out.push_str("'\\''");
        } else {
            out.push(c as char);
        }
    }
    out.push('\'');
}

pub struct GetoptApplet;
impl Applet for GetoptApplet {
    fn name(&self) -> &'static str {
        "getopt"
    }
    fn description(&self) -> &'static str {
        "Parse short+long options, print normalized quoted argv (util-linux subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut short_spec: Vec<u8> = Vec::new();
        let mut long_spec: Vec<u8> = Vec::new();
        let mut _name = "getopt".to_owned();
        let mut quiet = false;
        let mut test = false;
        let mut params: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"--" {
                i += 1;
                while i < args.len() {
                    params.push(&args[i]);
                    i += 1;
                }
                break;
            }
            if (b == b"-o" || b == b"--options") && i + 1 < args.len() {
                short_spec = ab(&args[i + 1]).to_vec();
                i += 2;
                continue;
            }
            if (b == b"-l" || b == b"--longoptions") && i + 1 < args.len() {
                long_spec = ab(&args[i + 1]).to_vec();
                i += 2;
                continue;
            }
            if (b == b"-n" || b == b"--name") && i + 1 < args.len() {
                _name = args[i + 1].to_string_lossy().into_owned();
                i += 2;
                continue;
            }
            if b == b"-q" || b == b"--quiet" {
                quiet = true;
                i += 1;
                continue;
            }
            if b == b"-T" || b == b"--test" {
                test = true;
                i += 1;
                continue;
            }
            if b == b"--options" || b.starts_with(b"--options=") {
                let v = if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    b[eq + 1..].to_vec()
                } else {
                    i += 1;
                    if i < args.len() {
                        ab(&args[i]).to_vec()
                    } else {
                        Vec::new()
                    }
                };
                short_spec = v;
                i += 1;
                continue;
            }
            if b.starts_with(b"-")
                && b.len() > 1
                && !b.starts_with(b"--")
                && short_spec.is_empty()
                && long_spec.is_empty()
                && params.is_empty()
            {
                short_spec = b[1..].to_vec();
                i += 1;
                continue;
            }
            params.push(&args[i]);
            i += 1;
        }
        if test {
            return Ok(4);
        }

        let mut argv: Vec<&[u8]> = Vec::new();
        let mut operands: Vec<&[u8]> = Vec::new();
        let mut seen_dd = false;
        for p in &params {
            if !seen_dd && ab(p) == b"--" {
                seen_dd = true;
                continue;
            }
            if seen_dd {
                operands.push(ab(p));
            } else {
                argv.push(ab(p));
            }
        }
        let spec = OptSpec {
            short: parse_short_spec(&short_spec),
            long: parse_long_spec(&long_spec),
        };
        let mut out = String::new();
        let mut rest: Vec<&[u8]> = Vec::new();
        let mut j = 0;
        let mut bad = false;
        while j < argv.len() {
            let tok = argv[j];
            if tok == b"--" {
                rest.extend_from_slice(&argv[j + 1..]);
                break;
            }
            if tok.starts_with(b"--") && tok.len() > 2 {
                let body = &tok[2..];
                let (lname, val) = match body.iter().position(|&c| c == b'=') {
                    Some(eq) => (&body[..eq], Some(&body[eq + 1..])),
                    None => (body, None),
                };
                let lstr = String::from_utf8_lossy(lname).into_owned();
                let matches: Vec<usize> = spec
                    .long
                    .iter()
                    .enumerate()
                    .filter(|(_, (n, _))| *n == lstr || n.starts_with(lstr.as_str()))
                    .map(|(k, _)| k)
                    .collect();
                let exact = spec.long.iter().position(|(n, _)| *n == lstr);
                let idx = if let Some(e) = exact {
                    Some(e)
                } else if matches.len() == 1 {
                    Some(matches[0])
                } else {
                    None
                };
                match idx {
                    None => {
                        if !quiet {
                            eprintln!("getopt: unrecognized option '--{lstr}'");
                        }
                        bad = true;
                        j += 1;
                    }
                    Some(k) => {
                        let (lname2, mode) = spec.long[k].clone();
                        match (mode, val) {
                            (0, Some(_)) => {
                                if !quiet {
                                    eprintln!(
                                        "getopt: option '--{lname2}' doesn't allow an argument"
                                    );
                                }
                                bad = true;
                            }
                            (0, None) => {
                                out.push_str(" --");
                                out.push_str(&lname2);
                            }
                            (_, Some(v)) => {
                                out.push_str(" --");
                                out.push_str(&lname2);
                                out.push(' ');
                                sh_quote(v, &mut out);
                            }
                            (_, None) => {
                                if j + 1 < argv.len() {
                                    j += 1;
                                    out.push_str(" --");
                                    out.push_str(&lname2);
                                    out.push(' ');
                                    sh_quote(argv[j], &mut out);
                                } else if mode == 1 {
                                    if !quiet {
                                        eprintln!(
                                            "getopt: option '--{lname2}' requires an argument"
                                        );
                                    }
                                    bad = true;
                                } else {
                                    out.push_str(" --");
                                    out.push_str(&lname2);
                                }
                            }
                        }
                        j += 1;
                    }
                }
                continue;
            }
            if tok.len() > 1 && tok[0] == b'-' {
                let mut k = 1;
                while k < tok.len() {
                    let c = tok[k];
                    match short_argmode(&spec.short, c) {
                        None => {
                            if !quiet {
                                eprintln!("getopt: invalid option -- '{}'", c as char);
                            }
                            bad = true;
                            k += 1;
                        }
                        Some(0) => {
                            out.push_str(" -");
                            out.push(c as char);
                            k += 1;
                        }
                        Some(m) => {
                            let rest_tok = &tok[k + 1..];
                            if !rest_tok.is_empty() {
                                out.push_str(" -");
                                out.push(c as char);
                                out.push(' ');
                                sh_quote(rest_tok, &mut out);
                                k = tok.len();
                            } else if j + 1 < argv.len() {
                                j += 1;
                                out.push_str(" -");
                                out.push(c as char);
                                out.push(' ');
                                sh_quote(argv[j], &mut out);
                                k = tok.len();
                            } else if m == 1 {
                                if !quiet {
                                    eprintln!(
                                        "getopt: option requires an argument -- '{}'",
                                        c as char
                                    );
                                }
                                bad = true;
                                k = tok.len();
                            } else {
                                out.push_str(" -");
                                out.push(c as char);
                                k = tok.len();
                            }
                        }
                    }
                }
                j += 1;
                continue;
            }

            rest.extend_from_slice(&argv[j..]);
            break;
        }
        if bad {
            return Ok(1);
        }
        out.push_str(" --");
        for r in rest.iter().chain(operands.iter()) {
            out.push(' ');
            sh_quote(r, &mut out);
        }
        println!("{out}");
        Ok(0)
    }
}

fn pw_lookup(name: &str) -> Option<(u32, u32)> {
    if let Ok(n) = name.parse::<u32>() {
        unsafe {
            let p = libc::getpwuid(n);
            if !p.is_null() {
                return Some(((*p).pw_uid, (*p).pw_gid));
            }
            return Some((n, n));
        }
    }
    let c = CString::new(name).ok()?;
    unsafe {
        let p = libc::getpwnam(c.as_ptr());
        if p.is_null() {
            return None;
        }
        Some(((*p).pw_uid, (*p).pw_gid))
    }
}

fn gr_lookup(name: &str) -> Option<u32> {
    if let Ok(n) = name.parse::<u32>() {
        return Some(n);
    }
    let c = CString::new(name).ok()?;
    unsafe {
        let g = libc::getgrnam(c.as_ptr());
        if g.is_null() {
            return None;
        }
        Some((*g).gr_gid)
    }
}

fn split_user_group(s: &str) -> (String, Option<String>) {
    if let Some((u, g)) = s.split_once([':', '.']) {
        (u.to_owned(), Some(g.to_owned()))
    } else {
        (s.to_owned(), None)
    }
}

fn exec_prog(prog: &OsString, args: &[OsString], uid: Option<u32>, gid: Option<u32>) -> i32 {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    if let Some(u) = uid {
        cmd.uid(u);
    }
    if let Some(g) = gid {
        cmd.gid(g);
    }
    let err = cmd.exec();
    eprintln!("{}: {err}", prog.to_string_lossy());
    1
}

pub struct EnvdirApplet;
impl Applet for EnvdirApplet {
    fn name(&self) -> &'static str {
        "envdir"
    }
    fn description(&self) -> &'static str {
        "Set env from dir files then exec prog (daemontools semantics)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("envdir: usage: envdir dir prog...");
            return Ok(1);
        }
        let dir = Path::new(&args[0]);
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("envdir: {}: {err}", dir.display());
                return Ok(1);
            }
        };
        let mut cmd = Command::new(&args[1]);
        cmd.args(&args[2..]);
        for ent in entries.flatten() {
            let fname = ent.file_name();
            let b = fname.as_bytes();
            if b.is_empty() || b.contains(&b'=') {
                continue;
            }
            let Ok(name) = std::str::from_utf8(b) else {
                continue;
            };
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            let data = match std::fs::read(ent.path()) {
                Ok(d) => d,
                Err(_) => continue,
            };
            if data.is_empty() {
                cmd.env_remove(name);
                continue;
            }

            let mut line = data.split(|&c| c == b'\n').next().unwrap_or(&[]).to_vec();
            for c in line.iter_mut() {
                if *c == 0 {
                    *c = b'\n';
                }
            }
            while line.last().is_some_and(|c| *c == b' ' || *c == b'\t') {
                line.pop();
            }
            cmd.env(name, OsStr::from_bytes(&line));
        }
        let err = cmd.exec();
        eprintln!("envdir: {}: {err}", args[1].to_string_lossy());
        Ok(1)
    }
}

pub struct EnvuidgidApplet;
impl Applet for EnvuidgidApplet {
    fn name(&self) -> &'static str {
        "envuidgid"
    }
    fn description(&self) -> &'static str {
        "Set $UID/$GID, drop privs to account, then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("envuidgid: usage: envuidgid account prog...");
            return Ok(1);
        }
        let acct = args[0].to_string_lossy().into_owned();
        let (uid, gid) = match pw_lookup(&acct) {
            Some(x) => x,
            None => {
                eprintln!("envuidgid: unknown account '{acct}'");
                return Ok(111);
            }
        };
        let mut cmd = Command::new(&args[1]);
        cmd.args(&args[2..]);
        cmd.env("UID", uid.to_string());
        cmd.env("GID", gid.to_string());
        cmd.uid(uid).gid(gid);
        let err = cmd.exec();
        eprintln!("envuidgid: {}: {err}", args[1].to_string_lossy());
        Ok(1)
    }
}

fn apply_rlimit(res: u32, n: u64) -> std::io::Result<()> {
    let mut cur: libc::rlimit = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrlimit(res, &mut cur) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let max = if cur.rlim_max == libc::RLIM_INFINITY || n < cur.rlim_max {
        n
    } else {
        cur.rlim_max
    };
    let _ = max;
    let new = libc::rlimit {
        rlim_cur: n.min(if cur.rlim_max == libc::RLIM_INFINITY {
            n
        } else {
            cur.rlim_max
        }),
        rlim_max: cur.rlim_max,
    };
    if unsafe { libc::setrlimit(res, &new) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub struct SoftlimitApplet;
impl Applet for SoftlimitApplet {
    fn name(&self) -> &'static str {
        "softlimit"
    }
    fn description(&self) -> &'static str {
        "Set soft rlimits (-a/-d/-m/-o/-s/-t/... ) then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut limits: Vec<(u32, u64)> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            let flag = if b.len() == 2 && b[0] == b'-' {
                Some(b[1])
            } else {
                None
            };
            let res = match flag {
                Some(b'a') => Some(libc::RLIMIT_AS),
                Some(b'c') => Some(libc::RLIMIT_CORE),
                Some(b'd') => Some(libc::RLIMIT_DATA),
                Some(b'f') => Some(libc::RLIMIT_FSIZE),
                Some(b'l') => Some(libc::RLIMIT_MEMLOCK),
                Some(b'o') => Some(libc::RLIMIT_NOFILE),
                Some(b'p') => Some(libc::RLIMIT_NPROC),
                Some(b'r') => Some(libc::RLIMIT_RSS),
                Some(b's') => Some(libc::RLIMIT_STACK),
                Some(b't') => Some(libc::RLIMIT_CPU),
                Some(b'm') => None,
                _ => None,
            };
            if flag == Some(b'm') && i + 1 < args.len() {
                let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                limits.push((libc::RLIMIT_DATA, n));
                limits.push((libc::RLIMIT_STACK, n));
                limits.push((libc::RLIMIT_RSS, n));
                i += 2;
                continue;
            }
            if let Some(r) = res {
                if i + 1 >= args.len() {
                    eprintln!("softlimit: -{} needs a value", flag.unwrap_or(b'?') as char);
                    return Ok(1);
                }
                let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                limits.push((r, n));
                i += 2;
                continue;
            }
            break;
        }
        if i >= args.len() {
            eprintln!("softlimit: missing prog");
            return Ok(1);
        }
        for (r, n) in &limits {
            if let Err(e) = apply_rlimit(*r, *n) {
                eprintln!("softlimit: setrlimit: {e}");
                return Ok(1);
            }
        }
        Ok(exec_prog(&args[i], &args[i + 1..], None, None))
    }
}

pub struct SetuidgidApplet;
impl Applet for SetuidgidApplet {
    fn name(&self) -> &'static str {
        "setuidgid"
    }
    fn description(&self) -> &'static str {
        "Drop privs to account (setgid+setuid) then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("setuidgid: usage: setuidgid account prog...");
            return Ok(1);
        }
        let acct = args[0].to_string_lossy().into_owned();
        let (user, group) = split_user_group(&acct);
        let (uid, mut gid) = match pw_lookup(&user) {
            Some(x) => x,
            None => {
                eprintln!("setuidgid: unknown account '{user}'");
                return Ok(111);
            }
        };
        if let Some(g) = group {
            match gr_lookup(&g) {
                Some(id) => gid = id,
                None => {
                    eprintln!("setuidgid: unknown group '{g}'");
                    return Ok(111);
                }
            }
        }
        Ok(exec_prog(&args[1], &args[2..], Some(uid), Some(gid)))
    }
}

pub struct ChpstApplet;
impl Applet for ChpstApplet {
    fn name(&self) -> &'static str {
        "chpst"
    }
    fn description(&self) -> &'static str {
        "Change state (-u/-U/-e/-/-/n/limits) then exec prog (runit subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut uid: Option<u32> = None;
        let mut gid: Option<u32> = None;
        let mut set_uidgid_env = false;
        let mut envdir: Option<PathBuf> = None;
        let mut root: Option<PathBuf> = None;
        let mut nice: Option<i32> = None;
        let mut verbose = false;
        let mut limits: Vec<(u32, u64)> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-U" {
                set_uidgid_env = true;
                i += 1;
            } else if b == b"-v" {
                verbose = true;
                i += 1;
            } else if b == b"-u" && i + 1 < args.len() {
                let (u, g) = split_user_group(&args[i + 1].to_string_lossy());
                match pw_lookup(&u) {
                    Some((uu, gg)) => {
                        uid = Some(uu);
                        gid = Some(gg);
                    }
                    None => {
                        eprintln!("chpst: unknown account '{u}'");
                        return Ok(111);
                    }
                }
                if let Some(g) = g {
                    match gr_lookup(&g) {
                        Some(id) => gid = Some(id),
                        None => {
                            eprintln!("chpst: unknown group '{g}'");
                            return Ok(111);
                        }
                    }
                }
                i += 2;
            } else if b == b"-e" && i + 1 < args.len() {
                envdir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if (b == b"-/" || b == b"--root") && i + 1 < args.len() {
                root = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if b == b"-n" && i + 1 < args.len() {
                nice = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 2;
            } else if b.len() == 2 && b[0] == b'-' && i + 1 < args.len() {
                let r = match b[1] {
                    b'a' => Some(libc::RLIMIT_AS),
                    b'd' => Some(libc::RLIMIT_DATA),
                    b'f' => Some(libc::RLIMIT_FSIZE),
                    b'c' => Some(libc::RLIMIT_CORE),
                    b'l' => Some(libc::RLIMIT_MEMLOCK),
                    b'o' => Some(libc::RLIMIT_NOFILE),
                    b'p' => Some(libc::RLIMIT_NPROC),
                    b'r' => Some(libc::RLIMIT_RSS),
                    b's' => Some(libc::RLIMIT_STACK),
                    b't' => Some(libc::RLIMIT_CPU),
                    b'm' => {
                        let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                            .parse()
                            .unwrap_or(0);
                        limits.push((libc::RLIMIT_DATA, n));
                        limits.push((libc::RLIMIT_STACK, n));
                        limits.push((libc::RLIMIT_RSS, n));
                        i += 2;
                        continue;
                    }
                    _ => None,
                };
                match r {
                    Some(res) => {
                        let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                            .parse()
                            .unwrap_or(0);
                        limits.push((res, n));
                        i += 2;
                    }
                    None => break,
                }
            } else {
                break;
            }
        }
        if i >= args.len() {
            eprintln!("chpst: missing prog");
            return Ok(1);
        }
        let mut cmd = Command::new(&args[i]);
        cmd.args(&args[i + 1..]);
        if let Some(d) = envdir {
            match std::fs::read_dir(&d) {
                Ok(entries) => {
                    for ent in entries.flatten() {
                        let b = ent.file_name();
                        let bb = b.as_bytes();
                        if bb.is_empty() || bb.contains(&b'=') {
                            continue;
                        }
                        let Ok(name) = std::str::from_utf8(bb) else {
                            continue;
                        };
                        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            continue;
                        }
                        match std::fs::read(ent.path()) {
                            Ok(data) if !data.is_empty() => {
                                let mut line =
                                    data.split(|&c| c == b'\n').next().unwrap_or(&[]).to_vec();
                                while line.last().is_some_and(|c| *c == b' ' || *c == b'\t') {
                                    line.pop();
                                }
                                cmd.env(name, OsStr::from_bytes(&line));
                            }
                            Ok(_) => {
                                cmd.env_remove(name);
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("chpst: {}: {e}", d.display());
                    return Ok(111);
                }
            }
        }
        if set_uidgid_env {
            if let (Some(u), Some(g)) = (uid, gid) {
                cmd.env("UID", u.to_string());
                cmd.env("GID", g.to_string());
            }
        }

        if root.is_some() || nice.is_some() {
            unsafe {
                cmd.pre_exec(move || {
                    if let Some(ref r) = root {
                        let bytes = r.as_os_str().as_bytes();
                        let c = CString::new(bytes).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad root")
                        })?;
                        if libc::chroot(c.as_ptr()) != 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                        if libc::chdir(c"/".as_ptr()) != 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    if let Some(n) = nice {
                        if libc::nice(n) == -1 {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    Ok(())
                });
            }
        }
        for (r, n) in &limits {
            if let Err(e) = apply_rlimit(*r, *n) {
                eprintln!("chpst: setrlimit: {e}");
                return Ok(1);
            }
        }
        if let Some(u) = uid {
            cmd.uid(u);
        }
        if let Some(g) = gid {
            cmd.gid(g);
        }
        if verbose {
            eprintln!("chpst: executing {}", args[i].to_string_lossy());
        }
        let err = cmd.exec();
        eprintln!("chpst: {}: {err}", args[i].to_string_lossy());
        Ok(1)
    }
}

fn supervise_pid(dir: &Path) -> Option<i32> {
    let data = std::fs::read(dir.join("supervise").join("pid")).ok()?;
    String::from_utf8_lossy(&data).trim().parse().ok()
}

fn supervise_stat(dir: &Path) -> Option<String> {
    let data = std::fs::read(dir.join("supervise").join("stat")).ok()?;
    Some(String::from_utf8_lossy(&data).trim().to_owned())
}

fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

fn sig_send(pid: i32, sig: i32) -> bool {
    unsafe { libc::kill(pid, sig) == 0 }
}

pub struct RunsvApplet;
impl Applet for RunsvApplet {
    fn name(&self) -> &'static str {
        "runsv"
    }
    fn description(&self) -> &'static str {
        "Supervise single service dir: run ./run, restart on exit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dir = if args.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(&args[0])
        };
        let run = dir.join("run");
        let finish = dir.join("finish");
        let _ = std::fs::create_dir_all(dir.join("supervise"));
        loop {
            if dir.join("down").exists() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            let mut child = match Command::new(&run).current_dir(&dir).spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("runsv: {}: {e}", run.display());
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }
            };
            let pid = child.id() as i32;
            let _ = std::fs::write(dir.join("supervise").join("pid"), pid.to_string());
            let _ = std::fs::write(dir.join("supervise").join("stat"), "run");
            let status = child.wait();
            let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
            if finish.exists() {
                let _ = Command::new(&finish)
                    .current_dir(&dir)
                    .arg(code.to_string())
                    .status();
            }
            let _ = std::fs::write(dir.join("supervise").join("stat"), "down");
            eprintln!("runsv: {pid} exited {code}, restarting");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

pub struct RunsvdirApplet;
impl Applet for RunsvdirApplet {
    fn name(&self) -> &'static str {
        "runsvdir"
    }
    fn description(&self) -> &'static str {
        "Scan dir for services, keep a runsv child on each"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dir = if args.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(&args[0])
        };
        let me = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("runsv"));
        let mut kids: Vec<(String, std::process::Child)> = Vec::new();
        loop {
            let mut seen: Vec<String> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for ent in entries.flatten() {
                    let p = ent.path();
                    if !p.is_dir() || !p.join("run").exists() {
                        continue;
                    }
                    let name = ent.file_name().to_string_lossy().into_owned();
                    seen.push(name.clone());
                    if kids.iter().any(|(n, _)| *n == name) {
                        continue;
                    }

                    let mut child = Command::new(&me).arg("runsv").arg(&p).spawn();
                    if child.is_err() {
                        child = Command::new("runsv").arg(&p).spawn();
                    }
                    match child {
                        Ok(c) => kids.push((name, c)),
                        Err(e) => eprintln!("runsvdir: {name}: {e}"),
                    }
                }
            }

            kids.retain_mut(|(n, c)| {
                if !seen.contains(n) {
                    let _ = c.kill();
                    let _ = c.wait();
                    return false;
                }
                match c.try_wait() {
                    Ok(Some(_)) => false,
                    Ok(None) => true,
                    Err(_) => false,
                }
            });
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn sv_status_one(dir: &OsString) -> i32 {
    let p = Path::new(dir);
    let stat = supervise_stat(p).unwrap_or_else(|| "down".to_owned());
    match supervise_pid(p) {
        Some(pid) if pid_alive(pid) => {
            println!("run: {}: (pid {pid})", dir.to_string_lossy());
            let _ = stat;
            0
        }
        _ => {
            println!("down: {}: 0s", dir.to_string_lossy());
            0
        }
    }
}

pub struct SvApplet;
impl Applet for SvApplet {
    fn name(&self) -> &'static str {
        "sv"
    }
    fn description(&self) -> &'static str {
        "Service control via supervise/ dir (status/up/down/once/signals subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut cmd = "status";
        let mut dirs: Vec<&OsString> = Vec::new();
        for a in args {
            let b = ab(a);
            if b == b"-v" {
                continue;
            }
            if !b.starts_with(b"-") || b == b"-1" || b == b"-2" {
                match (cmd, b) {
                    (
                        "status",
                        b"status" | b"up" | b"down" | b"once" | b"pause" | b"cont" | b"hup"
                        | b"alarm" | b"interrupt" | b"quit" | b"term" | b"kill" | b"1" | b"2"
                        | b"-1" | b"-2" | b"start" | b"stop" | b"restart" | b"shutdown"
                        | b"force-stop" | b"force-restart",
                    ) => {
                        cmd = match b {
                            b"status" => "status",
                            b"up" | b"start" => "up",
                            b"down" | b"stop" => "down",
                            b"once" => "once",
                            b"pause" => "pause",
                            b"cont" => "cont",
                            b"hup" => "hup",
                            b"alarm" => "alarm",
                            b"interrupt" => "interrupt",
                            b"quit" => "quit",
                            b"term" => "term",
                            b"kill" => "kill",
                            b"1" => "1",
                            b"2" => "2",
                            b"-1" => "1",
                            b"-2" => "2",
                            _ => "restart",
                        };
                    }
                    _ => dirs.push(a),
                }
            } else {
                eprintln!("sv: unknown option {}", b.escape_ascii());
                return Ok(1);
            }
        }
        if dirs.is_empty() {
            eprintln!("sv: missing service dir");
            return Ok(1);
        }
        let mut rc = 0;
        for d in dirs {
            let p = Path::new(d);
            match cmd {
                "status" => {
                    sv_status_one(d);
                }
                "up" => {
                    let _ = std::fs::remove_file(p.join("down"));
                    match supervise_pid(p) {
                        Some(pid) if pid_alive(pid) => {
                            sig_send(pid, libc::SIGCONT);
                        }
                        _ => eprintln!(
                            "sv: {}: no supervised pid (runsv will start it)",
                            d.to_string_lossy()
                        ),
                    }
                }
                "down" => {
                    let _ = std::fs::write(p.join("down"), "");
                    match supervise_pid(p) {
                        Some(pid) => {
                            if !sig_send(pid, libc::SIGTERM) {
                                eprintln!("sv: {}: signal failed", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                        None => {
                            eprintln!("sv: {}: no supervised pid", d.to_string_lossy());
                            rc = 1;
                        }
                    }
                }
                "once" => {
                    let _ = std::fs::remove_file(p.join("down"));
                    if supervise_pid(p).is_none() {
                        let run = p.join("run");
                        match Command::new(&run).current_dir(p).spawn() {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("sv: {}: {e}", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                    }
                }
                other => {
                    let sig = match other {
                        "pause" => libc::SIGSTOP,
                        "cont" => libc::SIGCONT,
                        "hup" => libc::SIGHUP,
                        "alarm" => libc::SIGALRM,
                        "interrupt" => libc::SIGINT,
                        "quit" => libc::SIGQUIT,
                        "term" | "restart" | "shutdown" | "force-stop" | "force-restart" => {
                            libc::SIGTERM
                        }
                        "kill" => libc::SIGKILL,
                        "1" => libc::SIGUSR1,
                        "2" => libc::SIGUSR2,
                        _ => 0,
                    };
                    match supervise_pid(p) {
                        Some(pid) => {
                            if other.starts_with("restart")
                                || other == "shutdown"
                                || other.starts_with("force")
                            {
                                let _ = std::fs::remove_file(p.join("down"));
                            }
                            if sig != 0 && !sig_send(pid, sig) {
                                eprintln!("sv: {}: signal failed", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                        None => {
                            eprintln!("sv: {}: no supervised pid", d.to_string_lossy());
                            rc = 1;
                        }
                    }
                }
            }
        }
        Ok(rc)
    }
}

pub struct SvcApplet;
impl Applet for SvcApplet {
    fn name(&self) -> &'static str {
        "svc"
    }
    fn description(&self) -> &'static str {
        "Send service commands (-u/-d/-o/-p/-c/-h/-t/-k/-x subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: Vec<u8> = Vec::new();
        let mut dirs: Vec<&OsString> = Vec::new();
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") && b.len() > 1 {
                flags.extend_from_slice(&b[1..]);
            } else {
                dirs.push(a);
            }
        }
        if flags.is_empty() || dirs.is_empty() {
            eprintln!("svc: usage: svc -u/-d/... servicedir...");
            return Ok(1);
        }
        let mut rc = 0;
        for d in dirs {
            let p = Path::new(d);
            for f in &flags {
                match f {
                    b'u' => {
                        let _ = std::fs::remove_file(p.join("down"));
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        }
                    }
                    b'd' => {
                        let _ = std::fs::write(p.join("down"), "");
                        match supervise_pid(p) {
                            Some(pid) => {
                                sig_send(pid, libc::SIGTERM);
                            }
                            None => {
                                eprintln!("svc: {}: no supervised pid", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                    }
                    b'o' => {
                        let _ = std::fs::remove_file(p.join("down"));
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        }
                    }
                    b'p' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGSTOP);
                        } else {
                            rc = 1;
                        }
                    }
                    b'c' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        } else {
                            rc = 1;
                        }
                    }
                    b'h' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGHUP);
                        } else {
                            rc = 1;
                        }
                    }
                    b'a' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGALRM);
                        } else {
                            rc = 1;
                        }
                    }
                    b'i' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGINT);
                        } else {
                            rc = 1;
                        }
                    }
                    b't' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGTERM);
                        } else {
                            rc = 1;
                        }
                    }
                    b'k' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGKILL);
                        } else {
                            rc = 1;
                        }
                    }
                    b'x' | b'X' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGTERM);
                        } else {
                            rc = 1;
                        }
                    }
                    _ => {
                        eprintln!("svc: unknown flag -{}", *f as char);
                        rc = 1;
                    }
                }
            }
        }
        Ok(rc)
    }
}

pub struct SvlogdApplet;
impl Applet for SvlogdApplet {
    fn name(&self) -> &'static str {
        "svlogd"
    }
    fn description(&self) -> &'static str {
        "Log stdin lines to dir/current with size rotation (-s/-n/-tt subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut max_size: u64 = 100_000;
        let mut keep: usize = 10;
        let mut tstamp = false;
        let mut dir: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-tt" || b == b"-t" {
                tstamp = true;
            } else if b.starts_with(b"-s") {
                let v = if b.len() > 2 {
                    String::from_utf8_lossy(&b[2..]).into_owned()
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].to_string_lossy().into_owned()
                } else {
                    String::new()
                };
                max_size = v.parse().unwrap_or(max_size);
            } else if b.starts_with(b"-n") {
                let v = if b.len() > 2 {
                    String::from_utf8_lossy(&b[2..]).into_owned()
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].to_string_lossy().into_owned()
                } else {
                    String::new()
                };
                keep = v.parse().unwrap_or(keep);
            } else if !b.starts_with(b"-") {
                dir = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }
        let dir = match dir {
            Some(d) => d,
            None => {
                eprintln!("svlogd: missing log dir");
                return Ok(1);
            }
        };
        if std::fs::create_dir_all(&dir).is_err() {
            eprintln!("svlogd: cannot create {}", dir.display());
            return Ok(1);
        }
        let cur = dir.join("current");
        let mut size = std::fs::metadata(&cur).map(|m| m.len()).unwrap_or(0);
        let stdin = std::io::stdin();
        let lines = std::io::BufRead::lines(stdin.lock());
        let mut f = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cur)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("svlogd: {e}");
                return Ok(1);
            }
        };
        for line in lines {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let entry = if tstamp {
                format!("{:?} {line}\n", std::time::SystemTime::now())
            } else {
                format!("{line}\n")
            };
            if f.write_all(entry.as_bytes()).is_err() {
                return Ok(1);
            }
            size += entry.len() as u64;
            if size >= max_size {
                drop(f);

                let oldest = dir.join(format!("current.{keep}"));
                let _ = std::fs::remove_file(&oldest);
                let mut k = keep;
                while k > 1 {
                    let _ = std::fs::rename(
                        dir.join(format!("current.{}", k - 1)),
                        dir.join(format!("current.{k}")),
                    );
                    k -= 1;
                }
                let _ = std::fs::rename(&cur, dir.join("current.1"));
                size = 0;
                f = match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&cur)
                {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("svlogd: {e}");
                        return Ok(1);
                    }
                };
            }
        }
        Ok(0)
    }
}

pub struct SetprivApplet;
impl Applet for SetprivApplet {
    fn name(&self) -> &'static str {
        "setpriv"
    }
    fn description(&self) -> &'static str {
        "Drop privileges (--reuid/--regid/--clear-groups subset) then exec"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut uid: Option<u32> = None;
        let mut gid: Option<u32> = None;
        let mut clear_groups = false;
        let mut prog_at: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            let s = String::from_utf8_lossy(b).into_owned();
            if s == "--" {
                prog_at = Some(i + 1);
                break;
            }
            if let Some(v) = s.strip_prefix("--reuid=") {
                uid = pw_lookup(v).map(|(u, _)| u).or_else(|| v.parse().ok());
            } else if s == "--reuid" && i + 1 < args.len() {
                let v = args[i + 1].to_string_lossy().into_owned();
                uid = pw_lookup(&v).map(|(u, _)| u).or_else(|| v.parse().ok());
                i += 1;
            } else if let Some(v) = s.strip_prefix("--regid=") {
                gid = gr_lookup(v).or_else(|| v.parse().ok());
            } else if s == "--regid" && i + 1 < args.len() {
                let v = args[i + 1].to_string_lossy().into_owned();
                gid = gr_lookup(&v).or_else(|| v.parse().ok());
                i += 1;
            } else if s == "--clear-groups" {
                clear_groups = true;
            } else if s == "--reset-env" {
            } else if s.starts_with("--") {
                eprintln!("setpriv: unsupported option {s}");
                return Ok(1);
            } else {
                prog_at = Some(i);
                break;
            }
            i += 1;
        }
        let at = match prog_at {
            Some(a) if a < args.len() => a,
            _ => {
                eprintln!("setpriv: missing prog");
                return Ok(1);
            }
        };
        let mut cmd = Command::new(&args[at]);
        cmd.args(&args[at + 1..]);
        unsafe {
            cmd.pre_exec(move || {
                if clear_groups && libc::setgroups(0, std::ptr::null()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if let Some(g) = gid {
                    if libc::setgid(g) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if let Some(u) = uid {
                    if libc::setuid(u) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        let err = cmd.exec();
        eprintln!("setpriv: {}: {err}", args[at].to_string_lossy());
        Ok(1)
    }
}
