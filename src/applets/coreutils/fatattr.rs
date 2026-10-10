use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;

applet!(
    FatattrApplet,
    "fatattr",
    "Show or change FAT attributes",
    run_fatattr
);
fn run_fatattr(args: &[OsString]) -> Result<i32> {
    const G: libc::c_ulong = 0x8004_7211;
    const S: libc::c_ulong = 0x4004_7211;
    let mut op: Vec<(u8, bool)> = Vec::new();
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b.len() > 1
            && (b[0] == b'+' || b[0] == b'-')
            && b[1..]
                .iter()
                .all(|&c| matches!(c, b'r' | b'h' | b's' | b'v' | b'd' | b'a'))
            && b.len() <= 7
        {
            let add = b[0] == b'+';
            for &c in &b[1..] {
                op.push((c, add));
            }
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("fatattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            files.push(a.clone());
        }
    }
    if files.is_empty() {
        eprintln!("fatattr: needs a file argument");
        return Ok(1);
    }
    use std::os::unix::io::AsRawFd;
    let bit = |c: u8| -> u32 {
        match c {
            b'r' => 0x01,
            b'h' => 0x02,
            b's' => 0x04,
            b'v' => 0x08,
            b'd' => 0x10,
            b'a' => 0x20,
            _ => 0,
        }
    };
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let fh = match File::open(Path::new(f)) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("fatattr: can't open '{}': {}", lossy(f), e);
                rc = 1;
                continue;
            }
        };
        let mut at: u32 = 0;
        if unsafe { libc::ioctl(fh.as_raw_fd(), G as _, &mut at) } < 0 {
            eprintln!(
                "fatattr: can't get attrs of '{}': {}",
                lossy(f),
                std::io::Error::last_os_error()
            );
            rc = 1;
            continue;
        }
        if op.is_empty() {
            let mut s = String::new();
            for c in *b"rhsvda" {
                s.push(if at & bit(c) != 0 { c as char } else { '-' });
            }
            let _ = writeln!(out, "{} {}", s, lossy(f));
        } else {
            for &(c, add) in &op {
                if add {
                    at |= bit(c);
                } else {
                    at &= !bit(c);
                }
            }
            if unsafe { libc::ioctl(fh.as_raw_fd(), S as _, &at) } < 0 {
                eprintln!(
                    "fatattr: can't set attrs of '{}': {}",
                    lossy(f),
                    std::io::Error::last_os_error()
                );
                rc = 1;
            }
        }
    }
    Ok(rc)
}

