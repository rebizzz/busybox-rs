use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self};
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStrExt;

pub struct SwapoffApplet;
impl Applet for SwapoffApplet {
    fn name(&self) -> &'static str {
        "swapoff"
    }
    fn description(&self) -> &'static str {
        "Stop swapping on specified device/file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut targets = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-a" {
                let swaps = fs::read_to_string("/proc/swaps").unwrap_or_default();
                for line in swaps.lines().skip(1) {
                    if let Some(dev) = line.split_whitespace().next() {
                        targets.push(OsString::from(dev));
                    }
                }
            } else if !b.starts_with(b"-") {
                targets.push(arg.clone());
            }
        }

        if targets.is_empty() {
            eprintln!("swapoff: [-a] [DEVICE]");
            return Ok(1);
        }

        let mut status = 0;
        for t in targets {
            let c_path = match CString::new(t.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let res = unsafe { libc::swapoff(c_path.as_ptr()) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("swapoff: {}: {}", t.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}
