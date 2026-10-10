use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::Path;

pub struct RpmHeader {
    tags: Vec<(u32, u32, Vec<u8>, u32)>,
}

pub fn parse_rpm_header(data: &[u8], off: usize) -> std::io::Result<(RpmHeader, usize)> {
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

pub struct RpmPkg {
    hdr: RpmHeader,
    payload: Vec<u8>,
    payload_comp: String,
}

pub fn parse_rpm(data: &[u8]) -> std::io::Result<RpmPkg> {
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

pub fn rpm_payload_cpio(pkg: &RpmPkg) -> std::io::Result<Vec<u8>> {
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
