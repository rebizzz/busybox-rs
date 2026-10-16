use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct FsyncApplet;

impl Applet for FsyncApplet {
    fn name(&self) -> &'static str {
        "fsync"
    }

    fn description(&self) -> &'static str {
        "Synchronize file state with storage"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: fsync [-d] FILE...");
            return Ok(1);
        }

        let mut data_only = false;
        let mut ret_code = 0;

        let mut files = Vec::new();
        for arg in &args[1..] {
            let b = arg.as_bytes();
            if b == b"-d" {
                data_only = true;
            } else {
                files.push(arg);
            }
        }

        for file_arg in files {
            let path = Path::new(file_arg);
            let f = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("fsync: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            let fd = f.as_raw_fd();
            let ret = if data_only {
                unsafe { libc::fdatasync(fd) }
            } else {
                unsafe { libc::fsync(fd) }
            };
            if ret < 0 {
                eprintln!("fsync: {}: {}", path.display(), io::Error::last_os_error());
                ret_code = 1;
            }
        }
        Ok(ret_code)
    }
}
