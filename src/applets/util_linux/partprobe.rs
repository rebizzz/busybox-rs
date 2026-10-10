use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct PartprobeApplet;
impl Applet for PartprobeApplet {
    fn name(&self) -> &'static str {
        "partprobe"
    }
    fn description(&self) -> &'static str {
        "Inform OS kernel of partition table changes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut devs = Vec::new();
        for arg in args {
            if !arg.as_bytes().starts_with(b"-") {
                devs.push(arg.clone());
            }
        }

        if devs.is_empty() {
            return Ok(0);
        }

        let mut status = 0;
        for dev in devs {
            let c_path = match CString::new(dev.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
            if fd < 0 {
                let err = io::Error::last_os_error();
                eprintln!("partprobe: {}: {}", dev.to_string_lossy(), err);
                status = 1;
                continue;
            }

            let res = unsafe { libc::ioctl(fd, BLKRRPART, 0) };
            unsafe { libc::close(fd) };

            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("partprobe: {}: {}", dev.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}
