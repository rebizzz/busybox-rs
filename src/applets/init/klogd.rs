#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct KlogdApplet;
impl Applet for KlogdApplet {
    fn name(&self) -> &'static str {
        "klogd"
    }
    fn description(&self) -> &'static str {
        "Kernel log daemon: forward /proc/kmsg to syslog (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_fork = false;
        let mut console_level: Option<u8> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out("klogd", "[-n] [-c N]", "Forward kernel messages to syslog");
            } else if b == b"-n" {
                no_fork = true;
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("klogd: option requires an argument -- 'c'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u8>().ok())
                {
                    Some(n) => console_level = Some(n),
                    None => {
                        eprintln!("klogd: invalid console level");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("klogd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        if let Some(n) = console_level {
            if unsafe { libc::klogctl(8, std::ptr::null_mut(), n as libc::c_int) } != 0 {
                eprintln!(
                    "klogd: cannot set console level: {}",
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
        }
        if !no_fork {
            let p = unsafe { libc::fork() };
            if p < 0 {
                eprintln!("klogd: fork: {}", io::Error::last_os_error());
                return Ok(1);
            }
            if p > 0 {
                return Ok(0);
            }
            unsafe {
                libc::setsid();
            }
        }

        klog_forward()
    }
}
