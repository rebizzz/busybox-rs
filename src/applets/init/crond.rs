#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct CrondApplet;
impl Applet for CrondApplet {
    fn name(&self) -> &'static str {
        "crond"
    }
    fn description(&self) -> &'static str {
        "Cron daemon: run scheduled jobs (subset: 5-field + @reboot skipped)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut foreground = false;
        let mut log_syslog = false;
        let mut log_level: u32 = 8;
        let mut log_file: Option<std::path::PathBuf> = None;
        let mut debug: u32 = 0;
        let mut dir = cron_dir();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "crond",
                    "[-f] [-b] [-S] [-l N] [-d N] [-L FILE] [-c DIR]",
                    "Run jobs from crontab files every minute",
                );
            } else if b == b"-f" {
                foreground = true;
            } else if b == b"-b" {
                foreground = false;
            } else if b == b"-S" {
                log_syslog = true;
            } else if b == b"-l" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crond: option requires an argument -- 'l'");
                    return Ok(1);
                }
                log_level = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8);
            } else if b == b"-L" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crond: option requires an argument -- 'L'");
                    return Ok(1);
                }
                log_file = Some(std::path::PathBuf::from(&args[i]));
            } else if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crond: option requires an argument -- 'd'");
                    return Ok(1);
                }
                debug = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                foreground = true;
            } else if b.starts_with(b"-d") && b.len() > 2 {
                debug = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                foreground = true;
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crond: option requires an argument -- 'c'");
                    return Ok(1);
                }
                dir = std::path::PathBuf::from(&args[i]);
            } else {
                eprintln!("crond: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        if !dir.is_dir() {
            eprintln!("crond: {}: no such directory", dir.display());
            return Ok(1);
        }
        if !foreground {
            let p = unsafe { libc::fork() };
            if p < 0 {
                eprintln!("crond: fork: {}", io::Error::last_os_error());
                return Ok(1);
            }
            if p > 0 {
                return Ok(0);
            }
            unsafe {
                libc::setsid();
            }
        }
        crond_loop(&dir, debug, log_level, log_syslog, log_file.as_deref())
    }
}
