use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct SuApplet;
impl Applet for SuApplet {
    fn name(&self) -> &'static str {
        "su"
    }
    fn description(&self) -> &'static str {
        "Switch user (subset: - -c -s; password needs crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut login_shell = false;
        let mut cmd: Option<Vec<u8>> = None;
        let mut shell: Option<Vec<u8>> = None;
        let mut user: Option<Vec<u8>> = None;
        let mut extra: Vec<Vec<u8>> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "su",
                    "[-] [-c CMD] [-s SHELL] [USER [ARGS...]]",
                    "Switch user identity",
                );
            } else if b == b"-" || b == b"-l" || b == b"--login" {
                login_shell = true;
            } else if b == b"-c" || b == b"--command" {
                i += 1;
                if i >= args.len() {
                    eprintln!("su: option requires an argument -- 'c'");
                    return Ok(1);
                }
                cmd = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-c") && b.len() > 2 {
                cmd = Some(b[2..].to_vec());
            } else if b == b"-s" || b == b"--shell" {
                i += 1;
                if i >= args.len() {
                    eprintln!("su: option requires an argument -- 's'");
                    return Ok(1);
                }
                shell = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-s") && b.len() > 2 {
                shell = Some(b[2..].to_vec());
            } else if b == b"--preserve-environment" {
            } else if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") {
                let mut ok = true;
                for &c in &b[1..] {
                    match c {
                        b'l' => login_shell = true,
                        b'p' | b'm' => {}
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    eprintln!("su: invalid option '{}'", String::from_utf8_lossy(b));
                    return Ok(1);
                }
            } else if b.starts_with(b"-") {
                eprintln!("su: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() {
                user = Some(b.to_vec());
            } else {
                extra.push(b.to_vec());
            }
            i += 1;
        }
        let _ = extra;
        let target = user.unwrap_or_else(|| b"root".to_vec());
        let ent = match lookup_user(&target) {
            Some(e) => e,
            None => {
                eprintln!("su: unknown user '{}'", String::from_utf8_lossy(&target));
                return Ok(1);
            }
        };

        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            let hash = shadow_hash(&target).unwrap_or_default();
            let pw = match read_secret(b"Password: ", false) {
                Some(p) => p,
                None => {
                    eprintln!("su: input error");
                    return Ok(1);
                }
            };
            match verify_password(&hash, &pw) {
                Ok(true) => {}
                Ok(false) => {
                    eprintln!("su: incorrect password");
                    return Ok(1);
                }
                Err(()) => return no_crypt("su"),
            }
        }
        if login_shell && !ent.home.is_empty() {
            if let Some(h) = cstr(&ent.home) {
                if unsafe { libc::chdir(h.as_ptr()) } != 0 {
                    eprintln!(
                        "su: cannot chdir to '{}'",
                        String::from_utf8_lossy(&ent.home)
                    );
                }
            }
        }
        become_and_exec(&ent, shell.as_deref(), login_shell, cmd.as_deref())
    }
}
