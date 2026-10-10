#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct WallApplet;
impl Applet for WallApplet {
    fn name(&self) -> &'static str {
        "wall"
    }
    fn description(&self) -> &'static str {
        "Broadcast a message to all logged-in users (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("wall", "[FILE]", "Broadcast FILE (or stdin) to all users");
            }
        }
        if args.len() > 1 {
            eprintln!("Usage: wall [FILE]");
            return Ok(1);
        }
        let msg: Vec<u8> = if args.is_empty() {
            let mut s = Vec::new();
            io::stdin().lock().read_to_end(&mut s)?;
            s
        } else {
            match fs::read(std::path::Path::new(&args[0])) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("wall: cannot read '{}': {}", args[0].to_string_lossy(), e);
                    return Ok(1);
                }
            }
        };
        if msg.is_empty() {
            eprintln!("wall: empty message");
            return Ok(1);
        }

        let mut hostn = [0 as libc::c_char; 65];

        unsafe {
            if libc::gethostname(hostn.as_mut_ptr(), hostn.len()) != 0 {
                hostn[0] = 0;
            }
        }
        let host = unsafe { std::ffi::CStr::from_ptr(hostn.as_ptr()).to_bytes().to_vec() };
        let me = current_username();
        let mut head: Vec<u8> = Vec::new();
        head.extend_from_slice(b"\r\nBroadcast message from ");
        head.extend_from_slice(&me);
        if !host.is_empty() {
            head.extend_from_slice(b"@");
            head.extend_from_slice(&host);
        }
        let mut now = Vec::new();
        fmt_time(&mut now, unsafe { libc::time(std::ptr::null_mut()) } as i64);
        head.extend_from_slice(b" (");
        head.extend_from_slice(&now);
        head.extend_from_slice(b"):\r\n\r\n");

        let mut body: Vec<u8> = Vec::with_capacity(msg.len() + 16);
        for &b in &msg {
            if b == b'\n' {
                body.extend_from_slice(b"\r\n");
            } else {
                body.push(b);
            }
        }
        if !body.ends_with(b"\r\n") {
            body.extend_from_slice(b"\r\n");
        }
        let utmp = std::env::var_os("BB_UTMP")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/var/run/utmp"));
        let entries = read_utmpx(Some(&utmp));
        let mut sent = 0;
        let mut failed = 0;
        for e in &entries {
            if e.typ != UT_USER_PROCESS || e.line.is_empty() {
                continue;
            }

            let mut tpath = std::path::PathBuf::from("/dev");
            tpath.push(String::from_utf8_lossy(&e.line).into_owned());
            let st = match fs::metadata(&tpath) {
                Ok(s) => s,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };
            {
                use std::os::unix::fs::PermissionsExt;
                if st.permissions().mode() & 0o020 == 0 {
                    continue;
                }
            }
            match OpenOptions::new().write(true).open(&tpath) {
                Ok(mut f) => {
                    if f.write_all(&head).is_ok() && f.write_all(&body).is_ok() {
                        sent += 1;
                    } else {
                        failed += 1;
                    }
                }
                Err(_) => failed += 1,
            }
        }
        if sent == 0 {
            eprintln!("wall: no users logged in or all ttys refused");
            return Ok(1);
        }
        let _ = failed;
        Ok(0)
    }
}
