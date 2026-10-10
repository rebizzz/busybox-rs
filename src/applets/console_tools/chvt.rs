use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ChvtApplet;

impl Applet for ChvtApplet {
    fn name(&self) -> &'static str {
        "chvt"
    }

    fn description(&self) -> &'static str {
        "Change foreground virtual terminal"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: chvt N");
            return Ok(1);
        }

        let vt_num = match parse_u32(args[1].as_bytes()) {
            Some(n) => n,
            None => {
                eprintln!("chvt: invalid number '{}'", args[1].to_string_lossy());
                return Ok(1);
            }
        };

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("chvt: {}", e);
                return Ok(1);
            }
        };

        let fd = console.as_raw_fd();
        let r1 = unsafe { libc::ioctl(fd, VT_ACTIVATE, vt_num as libc::c_ulong) };
        if r1 < 0 {
            eprintln!("chvt: VT_ACTIVATE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        let r2 = unsafe { libc::ioctl(fd, VT_WAITACTIVE, vt_num as libc::c_ulong) };
        if r2 < 0 {
            eprintln!("chvt: VT_WAITACTIVE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
