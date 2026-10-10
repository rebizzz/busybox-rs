#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct RunlevelApplet;
impl Applet for RunlevelApplet {
    fn name(&self) -> &'static str {
        "runlevel"
    }
    fn description(&self) -> &'static str {
        "Print previous and current runlevel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut file: Option<std::path::PathBuf> = None;
        for a in args {
            if is_help(a) {
                return help_out("runlevel", "[FILE]", "Print previous and current runlevel");
            } else if a.as_bytes().starts_with(b"-") {
                eprintln!(
                    "runlevel: invalid option '{}'",
                    String::from_utf8_lossy(a.as_bytes())
                );
                return Ok(1);
            } else if file.is_none() {
                file = Some(std::path::PathBuf::from(a));
            } else {
                eprintln!("runlevel: too many arguments");
                return Ok(1);
            }
        }
        let utmp = file
            .or_else(|| std::env::var_os("BB_UTMP").map(std::path::PathBuf::from))
            .unwrap_or_else(|| std::path::PathBuf::from("/var/run/utmp"));
        for e in read_utmpx(Some(&utmp)) {
            if e.typ == UT_RUN_LVL {
                let prev = (e.pid & 0xff) as u8;
                let curr = ((e.pid >> 8) & 0xff) as u8;
                let pc = if prev == 0 { b'N' } else { prev };
                let cc = if curr == 0 { b'N' } else { curr };
                let stdout = io::stdout();
                let mut o = stdout.lock();
                o.write_all(&[pc, b' ', cc, b'\n'])?;
                return Ok(0);
            }
        }
        eprintln!("runlevel: utmp: no runlevel record");
        Ok(1)
    }
}
