use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct FsfreezeApplet;
impl Applet for FsfreezeApplet {
    fn name(&self) -> &'static str {
        "fsfreeze"
    }
    fn description(&self) -> &'static str {
        "Flush and halt writes to mount point"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut freeze = None;
        let mut mountpoint = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"--freeze" || b == b"-f" {
                freeze = Some(true);
            } else if b == b"--unfreeze" || b == b"-u" {
                freeze = Some(false);
            } else if !b.starts_with(b"-") && mountpoint.is_none() {
                mountpoint = Some(arg.clone());
            }
        }

        let mnt = match (freeze, mountpoint) {
            (Some(f), Some(m)) => (f, m),
            _ => {
                eprintln!("fsfreeze: --freeze|--unfreeze MOUNTPOINT");
                return Ok(1);
            }
        };

        let c_path = match CString::new(mnt.1.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            eprintln!("fsfreeze: open {}: {}", mnt.1.to_string_lossy(), err);
            return Ok(1);
        }

        let op = if mnt.0 { FIFREEZE } else { FITHAW };
        let res = unsafe { libc::ioctl(fd, op, 0) };
        unsafe { libc::close(fd) };

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("fsfreeze: ioctl: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}
