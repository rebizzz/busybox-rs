use super::common::*;
use crate::core::Result;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

applet!(
    FindfsApplet,
    "findfs",
    "Find filesystem by label or UUID",
    run_findfs
);
fn run_findfs(args: &[OsString]) -> Result<i32> {
    if args.len() != 1 {
        eprintln!("findfs: needs exactly one LABEL=/UUID= argument");
        return Ok(1);
    }
    let q = lossy(&args[0]);
    let (k, vv) = match q.split_once('=') {
        Some(x) => x,
        None => {
            eprintln!("findfs: argument must be LABEL=<label> or UUID=<uuid>");
            return Ok(1);
        }
    };
    if k != "LABEL" && k != "UUID" {
        eprintln!("findfs: argument must be LABEL=<label> or UUID=<uuid>");
        return Ok(1);
    }
    for d in candidate_devs() {
        if let Some(vi) = probe_vol(d.as_os_str()) {
            let hit = if k == "LABEL" {
                vi.label == vv
            } else {
                vi.uuid.eq_ignore_ascii_case(vv)
            };
            if hit {
                println!("{}", lossy(&d));
                return Ok(0);
            }
        }
    }
    eprintln!("findfs: unable to resolve '{}'", q);
    Ok(1)
}

const F_GET: libc::c_ulong = 0x80086601;
const F_SET: libc::c_ulong = 0x40086602;
const ATTR_BITS: [(u8, libc::c_long); 13] = [
    (b's', 0x0000_0001),
    (b'u', 0x0000_0002),
    (b'c', 0x0000_0004),
    (b'S', 0x0000_0008),
    (b'i', 0x0000_0010),
    (b'a', 0x0000_0020),
    (b'A', 0x0000_0080),
    (b'd', 0x0000_0040),
    (b'D', 0x0001_0000),
    (b'E', 0x0000_0800),
    (b'e', 0x0008_0000),
    (b'I', 0x0000_1000),
    (b'j', 0x0000_4000),
];
const ATTR_ORDER: [u8; 13] = *b"sucSiaAdDEeIj";
fn bit_of(c: u8) -> Option<libc::c_long> {
    ATTR_BITS.iter().find(|&&(x, _)| x == c).map(|&(_, b)| b)
}
fn get_flags(p: &OsStr) -> std::io::Result<libc::c_long> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut fl: libc::c_long = 0;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_GET as _, &mut fl) };
    let _ = f;
    let _ = OsStr::from_bytes;
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(fl)
    }
}
fn set_flags(p: &OsStr, fl: libc::c_long) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut flm = fl;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_SET as _, &mut flm) };
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn flags_str(fl: libc::c_long) -> String {
    let mut s = String::with_capacity(20);
    for &c in &ATTR_ORDER {
        let b = bit_of(c).unwrap_or(0);
        s.push(if fl & b != 0 { c as char } else { '-' });
    }
    s
}
