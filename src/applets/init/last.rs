#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct LastApplet;
impl Applet for LastApplet {
    fn name(&self) -> &'static str {
        "last"
    }
    fn description(&self) -> &'static str {
        "Show login/logout history from wtmp (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut wtmp: Option<std::path::PathBuf> =
            std::env::var_os("BB_WTMP").map(std::path::PathBuf::from);
        let mut filter: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out("last", "[-HW] [-f FILE] [USER]", "Show wtmp login history");
            } else if b == b"-H" || b == b"-W" {
            } else if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("last: option requires an argument -- 'f'");
                    return Ok(1);
                }
                wtmp = Some(std::path::PathBuf::from(&args[i]));
            } else if b.starts_with(b"-") {
                eprintln!("last: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if filter.is_none() {
                filter = Some(b.to_vec());
            } else {
                eprintln!("last: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let wtmp = wtmp.unwrap_or_else(|| std::path::PathBuf::from("/var/log/wtmp"));
        if !wtmp.exists() {
            eprintln!("last: {}: no such file", wtmp.display());
            return Ok(1);
        }
        let entries = read_utmpx(Some(&wtmp));
        let stdout = io::stdout();
        let mut o = stdout.lock();
        let mut shown = 0;
        for e in &entries {
            if e.typ != UT_USER_PROCESS && e.typ != UT_BOOT_TIME && e.typ != UT_DEAD_PROCESS {
                continue;
            }
            if let Some(ref f) = filter {
                if &e.user != f {
                    continue;
                }
            }
            let what: &[u8] = match e.typ {
                UT_BOOT_TIME => b"system boot",
                UT_DEAD_PROCESS => b"shutdown",
                _ => &e.user,
            };
            o.write_all(what)?;
            o.write_all(b" ")?;
            o.write_all(if e.line.is_empty() { b"?" } else { &e.line })?;
            o.write_all(b" ")?;
            if !e.host.is_empty() {
                o.write_all(&e.host)?;
                o.write_all(b" ")?;
            }
            let mut t = Vec::new();
            fmt_time(&mut t, e.tv_sec);
            o.write_all(&t)?;
            o.write_all(b"\n")?;
            shown += 1;
        }
        if shown == 0 {
            eprintln!("last: no matching records");
            return Ok(1);
        }
        Ok(0)
    }
}
