#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct LogreadApplet;
impl Applet for LogreadApplet {
    fn name(&self) -> &'static str {
        "logread"
    }
    fn description(&self) -> &'static str {
        "Dump the syslog ring buffer (subset: file-backed ring)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut follow = false;
        for a in args {
            if is_help(a) {
                return help_out("logread", "[-f|-F]", "Dump the syslog ring buffer");
            } else if a.as_bytes() == b"-f" || a.as_bytes() == b"-F" {
                follow = true;
            } else {
                eprintln!(
                    "logread: invalid option '{}'",
                    String::from_utf8_lossy(a.as_bytes())
                );
                return Ok(1);
            }
        }
        let rp = ring_path();
        let data = match fs::read(&rp) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("logread: cannot read {}: {}", rp.display(), e);
                return Ok(1);
            }
        };
        {
            let stdout = io::stdout();
            let mut o = stdout.lock();
            o.write_all(&data)?;
            o.flush()?;
        }
        if follow {
            let mut pos = fs::metadata(&rp).map(|m| m.len()).unwrap_or(0);
            loop {
                let ts = libc::timespec {
                    tv_sec: 1,
                    tv_nsec: 0,
                };

                unsafe {
                    libc::nanosleep(&ts, std::ptr::null_mut());
                }
                let data = fs::read(&rp).unwrap_or_default();
                if (data.len() as u64) < pos {
                    pos = 0;
                }
                if (data.len() as u64) > pos {
                    let stdout = io::stdout();
                    let mut o = stdout.lock();
                    o.write_all(&data[pos as usize..])?;
                    o.flush()?;
                    pos = data.len() as u64;
                }
            }
        }
        Ok(0)
    }
}
