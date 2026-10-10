use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

pub struct AcpidApplet;

impl Applet for AcpidApplet {
    fn name(&self) -> &'static str {
        "acpid"
    }
    fn description(&self) -> &'static str {
        "Listen to ACPI events and spawn specific helper programs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut foreground = false;
        let mut event_file: Option<&Path> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-f" || b == b"--foreground" {
                foreground = true;
            } else if b == b"-e" || (!b.is_empty() && b[0] == b'-') {
            } else if event_file.is_none() {
                event_file = Some(Path::new(arg));
            }
        }

        let ev_path = event_file.unwrap_or_else(|| Path::new("/proc/acpi/event"));

        if !foreground {
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                eprintln!("acpid: fork failed");
                return Ok(1);
            }
            if pid > 0 {
                return Ok(0);
            }
            unsafe {
                libc::setsid();
            }
        }

        let file = match File::open(ev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("acpid: cannot open {}: {}", ev_path.display(), e);
                return Ok(1);
            }
        };

        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(|l| l.ok()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let handler = Path::new("/etc/acpi/handler.sh");
            if handler.exists() {
                let _ = std::process::Command::new(handler).args(&parts).status();
            }
        }

        Ok(0)
    }
}
