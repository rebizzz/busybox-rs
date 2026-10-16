use crate::applets::archival::common::*;
use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Read;
use std::path::Path;

pub struct CksumApplet;
impl Applet for CksumApplet {
    fn name(&self) -> &'static str {
        "cksum"
    }
    fn description(&self) -> &'static str {
        "Print POSIX CRC32 checksum and size"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let tab = build_crc_table();
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            let (crc, len) = cksum_stream(&tab, Path::new("-"))?;
            println!("{} {}", crc, len);
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            match cksum_stream(&tab, f) {
                Ok((crc, len)) => {
                    if f.as_os_str() == "-" {
                        println!("{} {}", crc, len);
                    } else {
                        println!("{} {} {}", crc, len, f.display());
                    }
                }
                Err(e) => {
                    eprintln!("cksum: {}: {}", f.display(), e);
                    rc = 1;
                }
            }
        }
        Ok(rc)
    }
}

fn cksum_stream(tab: &[u32; 256], path: &Path) -> Result<(u32, u64)> {
    let mut inp = open_or_stdin(path)?;
    let mut buf = [0u8; 8192];
    let mut crc = 0u32;
    let mut len = 0u64;
    loop {
        let n = inp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        crc = cksum_update(tab, crc, &buf[..n]);
        len += n as u64;
    }

    let mut l = len;
    while l > 0 {
        crc = cksum_update(tab, crc, &[((l & 0xff) as u8)]);
        l >>= 8;
    }
    Ok((!crc, len))
}
