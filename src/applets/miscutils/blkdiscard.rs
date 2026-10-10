use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct BlkdiscardApplet;

impl Applet for BlkdiscardApplet {
    fn name(&self) -> &'static str {
        "blkdiscard"
    }

    fn description(&self) -> &'static str {
        "Discard sectors on a block device"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: blkdiscard [-o offset] [-l length] DEVICE");
            return Ok(1);
        }

        let mut offset = 0u64;
        let mut length = 0u64;
        let mut dev = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-o" && i + 1 < args.len() {
                i += 1;
                offset = parse_u64(args[i].as_bytes()).unwrap_or(0);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                length = parse_u64(args[i].as_bytes()).unwrap_or(0);
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let dev_path = match dev {
            Some(d) => Path::new(d),
            None => {
                eprintln!("blkdiscard: no device specified");
                return Ok(1);
            }
        };

        let f = match OpenOptions::new().read(true).write(true).open(dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("blkdiscard: {}: {}", dev_path.display(), e);
                return Ok(1);
            }
        };

        if length == 0 {
            let mut sz: u64 = 0;
            let ret = unsafe {
                libc::ioctl(
                    f.as_raw_fd(),
                    BLKGETSIZE64,
                    &mut sz as *mut u64 as *mut libc::c_void,
                )
            };
            if ret == 0 && sz > offset {
                length = sz - offset;
            }
        }

        let range: [u64; 2] = [offset, length];
        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                BLKDISCARD,
                range.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "blkdiscard: BLKDISCARD failed on {}: {}",
                dev_path.display(),
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}
