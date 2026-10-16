use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub struct ChrootApplet;
impl Applet for ChrootApplet {
    fn name(&self) -> &'static str {
        "chroot"
    }
    fn description(&self) -> &'static str {
        "Run command with a different root directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut root: Option<&[u8]> = None;
        let mut at: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--userspec" {
                i += 1;
            } else if b.starts_with(b"--userspec=") {
            } else if b.first() == Some(&b'-') {
                eprintln!("chroot: unknown option");
                return Ok(1);
            } else if root.is_none() {
                root = Some(b);
            } else {
                at = Some(i);
                break;
            }
            i += 1;
        }
        let root = match root {
            Some(r) => r,
            None => {
                eprintln!("usage: chroot NEWROOT [COMMAND...]");
                return Ok(1);
            }
        };
        use std::ffi::CString;
        let rc = CString::new(root).unwrap_or_else(|_| CString::new("/").unwrap());
        if unsafe { libc::chroot(rc.as_ptr()) } != 0 {
            eprintln!(
                "chroot: can't change root directory to '{}': {}",
                String::from_utf8_lossy(root),
                strerror_last()
            );
            return Ok(1);
        }
        if unsafe { libc::chdir(c"/".as_ptr()) } != 0 {
            eprintln!("chroot: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        match at {
            Some(p) => {
                let cmd: Vec<OsString> = args[p..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
            None => {
                let sh = OsString::from_vec(b"/bin/sh".to_vec());
                let dash = OsString::from_vec(b"-i".to_vec());
                Ok(exec_prog(b"/bin/sh", &[dash, sh]))
            }
        }
    }
}
