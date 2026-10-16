#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct SyslogdApplet;
impl Applet for SyslogdApplet {
    fn name(&self) -> &'static str {
        "syslogd"
    }
    fn description(&self) -> &'static str {
        "Syslog daemon: receive /dev/log, write log file + ring (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_fork = false;
        let mut outfile = std::env::var_os("BB_SYSLOG_OUT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/var/log/messages"));
        let mut ring_cap: usize = 16 * 1024;
        let mut mark_min: u64 = 20;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "syslogd",
                    "[-n] [-O FILE] [-C[size]] [-m MIN]",
                    "Receive /dev/log datagrams into a log file and ring buffer",
                );
            } else if b == b"-n" || b == b"-f" {
                no_fork = true;
            } else if b == b"-O" {
                i += 1;
                if i >= args.len() {
                    eprintln!("syslogd: option requires an argument -- 'O'");
                    return Ok(1);
                }
                outfile = std::path::PathBuf::from(&args[i]);
            } else if b.starts_with(b"-O") && b.len() > 2 {
                outfile = std::path::PathBuf::from(std::ffi::OsStr::from_bytes(&b[2..]));
            } else if b == b"-C" {
                ring_cap = 16 * 1024;
            } else if b.starts_with(b"-C") && b.len() > 2 {
                match std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                {
                    Some(n) => ring_cap = n,
                    None => {
                        eprintln!("syslogd: invalid -C size");
                        return Ok(1);
                    }
                }
            } else if b == b"-m" {
                i += 1;
                if i >= args.len() {
                    eprintln!("syslogd: option requires an argument -- 'm'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    Some(n) => mark_min = n,
                    None => {
                        eprintln!("syslogd: invalid mark interval");
                        return Ok(1);
                    }
                }
            } else if b == b"-R" || b == b"-L" || b == b"-S" || b == b"-s" || b == b"-b" {
                eprintln!(
                    "syslogd: option '{}' not supported in this build",
                    String::from_utf8_lossy(b)
                );
                return Ok(1);
            } else {
                eprintln!("syslogd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        if !no_fork {
            let p = unsafe { libc::fork() };
            if p < 0 {
                eprintln!("syslogd: fork: {}", io::Error::last_os_error());
                return Ok(1);
            }
            if p > 0 {
                return Ok(0);
            }
            unsafe {
                libc::setsid();
            }
        }
        syslog_loop(&outfile, ring_cap, mark_min)
    }
}
