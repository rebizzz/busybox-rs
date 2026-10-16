use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::io::AsRawFd;

pub struct UbidetachApplet;
impl Applet for UbidetachApplet {
    fn name(&self) -> &'static str {
        "ubidetach"
    }
    fn description(&self) -> &'static str {
        "Detach UBI device (UBI_CTRL_IOCDET; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ubi: Option<i32> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"-d" || b == b"--devn") && i + 1 < args.len() {
                ubi = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 1;
            }
            i += 1;
        }
        let ubi = match ubi {
            Some(u) => u,
            None => {
                eprintln!("ubidetach: missing -d UBI number");
                return Ok(1);
            }
        };
        let (f, dev) = match ubi_ctrl_open(None) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("ubidetach: /dev/ubi_ctrl: {e}");
                return Ok(1);
            }
        };
        let code = iow(b'O', 65, size_of::<i32>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), code, &ubi) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}
