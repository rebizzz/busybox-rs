#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct GettyApplet;
impl Applet for GettyApplet {
    fn name(&self) -> &'static str {
        "getty"
    }
    fn description(&self) -> &'static str {
        "Open a tty, prompt for login name, exec login (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut issue: Option<std::path::PathBuf> = None;
        let mut login_prog = std::env::var_os("BB_LOGIN_PROG")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/bin/login"));
        let mut no_issue = false;
        let mut no_prompt = false;
        let mut timeout: u64 = 0;
        let mut pos: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "getty",
                    "[-hLmiwn] [-t SEC] [-f ISSUE] [-l LOGIN] BAUD TTY [TERM]",
                    "Prompt for a login name on a tty and exec login",
                );
            } else if b == b"-i" {
                no_issue = true;
            } else if b == b"-n" {
                no_prompt = true;
            } else if b == b"-h" || b == b"-L" || b == b"-m" || b == b"-w" {
            } else if b == b"-t" {
                i += 1;
                if i >= args.len() {
                    eprintln!("getty: option requires an argument -- 't'");
                    return Ok(1);
                }
                timeout = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if b.starts_with(b"-t") && b.len() > 2 {
                timeout = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("getty: option requires an argument -- 'f'");
                    return Ok(1);
                }
                issue = Some(std::path::PathBuf::from(&args[i]));
            } else if b == b"-l" {
                i += 1;
                if i >= args.len() {
                    eprintln!("getty: option requires an argument -- 'l'");
                    return Ok(1);
                }
                login_prog = std::path::PathBuf::from(&args[i]);
            } else if b.starts_with(b"-") {
                eprintln!("getty: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(&args[i]);
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("Usage: getty [-i] [-n] [-t SEC] [-f ISSUE] [-l LOGIN] BAUD TTY [TERM]");
            return Ok(1);
        }
        let baud_s = String::from_utf8_lossy(pos[0].as_bytes()).into_owned();
        let first_baud: u32 = baud_s.split(',').next().unwrap_or("").parse().unwrap_or(0);
        let baud = match baud_const(first_baud) {
            Some(b) => b,
            None => {
                eprintln!("getty: bad baud rate '{}'", baud_s);
                return Ok(1);
            }
        };
        let tty = std::path::PathBuf::from(&pos[1]);
        getty_run(
            &tty,
            baud,
            issue.as_deref(),
            &login_prog,
            no_issue,
            no_prompt,
            timeout,
        )
    }
}
