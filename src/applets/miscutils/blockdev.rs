use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct BlockdevApplet;

impl Applet for BlockdevApplet {
    fn name(&self) -> &'static str {
        "blockdev"
    }

    fn description(&self) -> &'static str {
        "Control block devices"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 3 {
            eprintln!("Usage: blockdev --setro|--setrw|--getro|--getss|--getsize|--getsize64|--getra|--setra N|--flushbufs DEVICE");
            return Ok(1);
        }

        let cmd = args[1].as_bytes();
        let (dev_arg, set_val) = if cmd == b"--setra" {
            if args.len() < 4 {
                eprintln!("Usage: blockdev --setra N DEVICE");
                return Ok(1);
            }
            (&args[3], parse_u32(args[2].as_bytes()).unwrap_or(0))
        } else {
            (&args[2], 0)
        };

        let path = Path::new(dev_arg);
        let f = match OpenOptions::new().read(true).write(true).open(path) {
            Ok(f) => f,
            Err(_) => match OpenOptions::new().read(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("blockdev: {}: {}", path.display(), e);
                    return Ok(1);
                }
            },
        };

        let fd = f.as_raw_fd();
        match cmd {
            b"--setro" => {
                let val: libc::c_int = 1;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROSET,
                        &val as *const libc::c_int as *const libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--setrw" => {
                let val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROSET,
                        &val as *const libc::c_int as *const libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--getro" => {
                let mut val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROGET,
                        &mut val as *mut libc::c_int as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getss" => {
                let mut val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKSSZGET,
                        &mut val as *mut libc::c_int as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getsize" => {
                let mut val: libc::c_ulong = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKGETSIZE,
                        &mut val as *mut libc::c_ulong as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getsize64" => {
                let mut val: u64 = 0;
                let ret = unsafe {
                    libc::ioctl(fd, BLKGETSIZE64, &mut val as *mut u64 as *mut libc::c_void)
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getra" => {
                let mut val: libc::c_long = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKRAGET,
                        &mut val as *mut libc::c_long as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--setra" => {
                let val = set_val as libc::c_ulong;
                let ret = unsafe { libc::ioctl(fd, BLKRASET, val) };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--flushbufs" => {
                let ret = unsafe { libc::ioctl(fd, BLKFLSBUF, 0 as libc::c_ulong) };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            _ => {
                eprintln!("blockdev: unknown command '{}'", args[1].to_string_lossy());
                return Ok(1);
            }
        }
        Ok(0)
    }
}
