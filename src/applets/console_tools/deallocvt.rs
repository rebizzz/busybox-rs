use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct DeallocvtApplet;

impl Applet for DeallocvtApplet {
    fn name(&self) -> &'static str {
        "deallocvt"
    }

    fn description(&self) -> &'static str {
        "Deallocate unused virtual consoles"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let vt_num = if args.len() > 1 {
            match parse_u32(args[1].as_bytes()) {
                Some(n) => n,
                None => {
                    eprintln!("deallocvt: invalid number");
                    return Ok(1);
                }
            }
        } else {
            0
        };

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("deallocvt: {}", e);
                return Ok(1);
            }
        };

        let ret =
            unsafe { libc::ioctl(console.as_raw_fd(), VT_DISALLOCATE, vt_num as libc::c_ulong) };
        if ret < 0 {
            eprintln!("deallocvt: failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
