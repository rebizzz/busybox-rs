use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self};
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStrExt;

pub struct SwaponApplet;
impl Applet for SwaponApplet {
    fn name(&self) -> &'static str {
        "swapon"
    }
    fn description(&self) -> &'static str {
        "Start swapping on specified device/file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: libc::c_int = 0;
        let mut targets = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" {
                let fstab = fs::read_to_string("/etc/fstab").unwrap_or_default();
                for line in fstab.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[2] == "swap" {
                        targets.push(OsString::from(parts[0]));
                    }
                }
            } else if b == b"-p" && i + 1 < args.len() {
                if let Ok(prio) = args[i + 1].to_string_lossy().parse::<libc::c_int>() {
                    flags |= 0x8000 | (prio & 0x7fff);
                }
                i += 2;
                continue;
            } else if b == b"-d" {
                flags |= 0x10000;
            } else if !b.starts_with(b"-") {
                targets.push(args[i].clone());
            }
            i += 1;
        }

        if targets.is_empty() {
            eprintln!("swapon: [-a] [DEVICE]");
            return Ok(1);
        }

        let mut status = 0;
        for t in targets {
            let c_path = match CString::new(t.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let res = unsafe { libc::swapon(c_path.as_ptr(), flags) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("swapon: {}: {}", t.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}
