use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct RaidautorunApplet;

const RAID_AUTORUN: libc::c_ulong = 0x914;

impl Applet for RaidautorunApplet {
    fn name(&self) -> &'static str {
        "raidautorun"
    }
    fn description(&self) -> &'static str {
        "Tell kernel to automatically configure RAID arrays"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dev = if !args.is_empty() {
            Path::new(&args[0])
        } else {
            Path::new("/dev/md0")
        };

        let f = match OpenOptions::new().read(true).write(true).open(dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("raidautorun: {}: {}", dev.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(f.as_raw_fd(), RAID_AUTORUN, 0) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            eprintln!("raidautorun: failed: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}
