use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct SuloginApplet;
impl Applet for SuloginApplet {
    fn name(&self) -> &'static str {
        "sulogin"
    }
    fn description(&self) -> &'static str {
        "Single-user login (subset: root password, no crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut timeout: u64 = 0;
        let mut tty: Option<&OsString> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out("sulogin", "[-t SEC] [TTY]", "Single-user root login");
            } else if b == b"-t" {
                i += 1;
                if i >= args.len() {
                    eprintln!("sulogin: option requires an argument -- 't'");
                    return Ok(1);
                }
                timeout = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if b == b"-p" {
            } else if b.starts_with(b"-") {
                eprintln!("sulogin: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if tty.is_none() {
                tty = Some(&args[i]);
            } else {
                eprintln!("sulogin: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let _ = timeout;
        let _ = tty;
        let root = lookup_user(b"root").unwrap_or(PasswdEnt {
            name: b"root".to_vec(),
            pass: b"x".to_vec(),
            uid: 0,
            gid: 0,
            gecos: Vec::new(),
            home: b"/root".to_vec(),
            shell: b"/bin/sh".to_vec(),
        });
        let hash = shadow_hash(b"root").unwrap_or_default();
        let pw = match read_secret(
            b"Give root password for maintenance (or press Ctrl-D to continue): ",
            false,
        ) {
            Some(p) => p,
            None => {
                if hash.is_empty() {
                    return become_and_exec(&root, Some(b"/bin/sh"), false, None);
                }
                eprintln!("sulogin: no password given");
                return Ok(1);
            }
        };
        match verify_password(&hash, &pw) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("sulogin: incorrect password");
                return Ok(1);
            }
            Err(()) => return no_crypt("sulogin"),
        }
        become_and_exec(&root, Some(b"/bin/sh"), false, None)
    }
}
