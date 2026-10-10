use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct FdflushApplet;

impl Applet for FdflushApplet {
    fn name(&self) -> &'static str {
        "fdflush"
    }

    fn description(&self) -> &'static str {
        "Flush floppy disk buffer cache"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dev = if args.len() > 1 {
            &args[1]
        } else {
            eprintln!("Usage: fdflush DEVICE");
            return Ok(1);
        };

        let path = Path::new(dev);
        let f = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fdflush: {}: {}", path.display(), e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, FDFLUSH, 0 as libc::c_ulong) };
        if ret < 0 {
            let ret2 = unsafe { libc::ioctl(fd, BLKFLSBUF, 0 as libc::c_ulong) };
            if ret2 < 0 {
                eprintln!(
                    "fdflush: failed on {}: {}",
                    path.display(),
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
        }
        Ok(0)
    }
}
