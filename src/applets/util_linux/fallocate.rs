use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct FallocateApplet;
impl Applet for FallocateApplet {
    fn name(&self) -> &'static str {
        "fallocate"
    }
    fn description(&self) -> &'static str {
        "Preallocate space for a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut len: Option<u64> = None;
        let mut off: u64 = 0;
        let mut mode: i32 = 0;
        let mut file: Option<&[u8]> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" || b == b"--length" {
                i += 1;
                if i >= args.len() {
                    eprintln!("fallocate: -l needs a length");
                    return Ok(1);
                }
                match parse_u64_suffix(args[i].as_bytes()) {
                    Some(v) => len = Some(v),
                    None => {
                        eprintln!("fallocate: invalid length");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"--length=") {
                match parse_u64_suffix(&b[9..]) {
                    Some(v) => len = Some(v),
                    None => {
                        eprintln!("fallocate: invalid length");
                        return Ok(1);
                    }
                }
            } else if b == b"-o" || b == b"--offset" {
                i += 1;
                if i >= args.len() {
                    eprintln!("fallocate: -o needs an offset");
                    return Ok(1);
                }
                match parse_u64_suffix(args[i].as_bytes()) {
                    Some(v) => off = v,
                    None => {
                        eprintln!("fallocate: invalid offset");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"--offset=") {
                match parse_u64_suffix(&b[9..]) {
                    Some(v) => off = v,
                    None => {
                        eprintln!("fallocate: invalid offset");
                        return Ok(1);
                    }
                }
            } else if b == b"-n" || b == b"--keep-size" {
                mode |= libc::FALLOC_FL_KEEP_SIZE;
            } else if b.first() == Some(&b'-') {
                eprintln!("fallocate: unknown option");
                return Ok(1);
            } else if file.is_none() {
                file = Some(b);
            } else {
                eprintln!("fallocate: too many files");
                return Ok(1);
            }
            i += 1;
        }
        let (len, file) = match (len, file) {
            (Some(l), Some(f)) => (l, f),
            _ => {
                eprintln!("usage: fallocate [-o OFFSET] -l LENGTH [-n] FILE");
                return Ok(1);
            }
        };
        use std::ffi::CString;
        let cf = match CString::new(file) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("fallocate: bad filename");
                return Ok(1);
            }
        };
        let fd = unsafe { libc::open(cf.as_ptr(), libc::O_RDWR | libc::O_CREAT, 0o644) };
        if fd < 0 {
            eprintln!(
                "fallocate: '{}': {}",
                String::from_utf8_lossy(file),
                std::io::Error::last_os_error()
            );
            return Ok(1);
        }
        let r = unsafe { libc::fallocate(fd, mode, off as libc::off_t, len as libc::off_t) };
        unsafe { libc::close(fd) };
        if r != 0 {
            eprintln!("fallocate: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
