use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct FreeramdiskApplet;

impl Applet for FreeramdiskApplet {
    fn name(&self) -> &'static str {
        "freeramdisk"
    }

    fn description(&self) -> &'static str {
        "Free memory held by a ramdisk"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: freeramdisk DEVICE");
            return Ok(1);
        }

        let path = Path::new(&args[1]);
        let f = match OpenOptions::new().write(true).open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("freeramdisk: {}: {}", path.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(f.as_raw_fd(), BLKFLSBUF, 0 as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "freeramdisk: BLKFLSBUF failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}
