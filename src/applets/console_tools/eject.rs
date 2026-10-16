use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::unix::ffi::OsStrExt;

pub struct EjectApplet;

impl Applet for EjectApplet {
    fn name(&self) -> &'static str {
        "eject"
    }

    fn description(&self) -> &'static str {
        "Eject removable media"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut close_tray = false;
        let mut dev_path = "/dev/cdrom";

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" || b == b"-c" {
                close_tray = true;
            } else if !b.starts_with(b"-") {
                if let Ok(s) = std::str::from_utf8(b) {
                    dev_path = s;
                }
            }
            i += 1;
        }

        let f = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(dev_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("eject: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let cmd = if close_tray {
            CDROMCLOSETRAY
        } else {
            CDROMEJECT
        };
        let ret = unsafe { libc::ioctl(f.as_raw_fd(), cmd, 0 as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "eject: failed on {}: {}",
                dev_path,
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}
