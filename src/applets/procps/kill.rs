use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct KillApplet;
impl Applet for KillApplet {
    fn name(&self) -> &'static str {
        "kill"
    }
    fn description(&self) -> &'static str {
        "Send a signal to processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut pids: Vec<i32> = Vec::new();
        let mut i = 0;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" {
                list_signals(&mut out)?;
                out.flush()?;
                return Ok(0);
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("kill: -s needs a signal argument");
                    return Ok(1);
                }
                match sig_from_name(args[i].as_bytes()) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("kill: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("kill: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                let s = args[i].to_string_lossy();
                match s.parse::<i32>() {
                    Ok(pid) => pids.push(pid),
                    Err(_) => {
                        eprintln!("kill: '{}': invalid pid", s);
                        return Ok(1);
                    }
                }
            }
            i += 1;
        }
        if pids.is_empty() {
            eprintln!("kill: usage: kill [-l] [-SIG] PID...");
            return Ok(1);
        }
        let mut rc = 0;
        for pid in pids {
            if unsafe { libc::kill(pid, sig) } != 0 {
                eprintln!("kill: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}
