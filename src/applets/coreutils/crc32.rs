use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn crc_tab() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (i, slot) in t.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    t
}
applet!(Crc32Applet, "crc32", "Print CRC32 checksums", run_crc32);
fn run_crc32(args: &[OsString]) -> Result<i32> {
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"--" {
            continue;
        }
        if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("crc32: invalid option '{}'", lossy(a));
            return Ok(1);
        }
        files.push(a.clone());
    }
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let tab = crc_tab();
    let mut rc = 0;
    let mut out = wlock();
    let mut line = Vec::with_capacity(64);
    for f in &files {
        let mut crc = 0xffff_ffffu32;
        let mut len = 0u64;
        let r = if f.as_os_str().as_bytes() == b"-" {
            let mut inp = std::io::stdin().lock();
            let mut buf = [0u8; 8192];
            loop {
                match inp.read(&mut buf) {
                    Ok(0) => break Ok(()),
                    Ok(n) => {
                        len += n as u64;
                        for &x in &buf[..n] {
                            crc = tab[((crc ^ x as u32) & 0xff) as usize] ^ (crc >> 8);
                        }
                        continue;
                    }
                    Err(e) => break Err(e),
                }
            }
        } else {
            match File::open(Path::new(f)) {
                Ok(mut fh) => {
                    let mut buf = [0u8; 8192];
                    loop {
                        match fh.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => {
                                len += n as u64;
                                for &x in &buf[..n] {
                                    crc = tab[((crc ^ x as u32) & 0xff) as usize] ^ (crc >> 8);
                                }
                                continue;
                            }
                            Err(e) => break Err(e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("crc32: can't open '{}': {}", lossy(f), e);
                    rc = 1;
                    continue;
                }
            }
        };
        if let Err(e) = r {
            eprintln!("crc32: {}: {}", lossy(f), e);
            rc = 1;
            continue;
        }
        crc ^= 0xffff_ffff;
        line.clear();
        let _ = writeln!(line, "{:08x}\t{}\t{}", crc, len, lossy(f));
        let _ = out.write_all(&line);
    }
    Ok(rc)
}

