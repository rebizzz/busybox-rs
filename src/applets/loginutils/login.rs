use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct LoginApplet;
impl Applet for LoginApplet {
    fn name(&self) -> &'static str {
        "login"
    }
    fn description(&self) -> &'static str {
        "Log in as a user (subset: passwd/shadow auth, no crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut preserve = false;
        let mut force_user: Option<Vec<u8>> = None;
        let mut host: Option<Vec<u8>> = None;
        let mut user: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "login",
                    "[-p] [-h HOST] [-f USER] [USER]",
                    "Log in as a user",
                );
            } else if b == b"-p" {
                preserve = true;
            } else if b == b"-h" {
                i += 1;
                if i >= args.len() {
                    eprintln!("login: option requires an argument -- 'h'");
                    return Ok(1);
                }
                host = Some(args[i].as_bytes().to_vec());
            } else if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("login: option requires an argument -- 'f'");
                    return Ok(1);
                }
                force_user = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-") {
                eprintln!("login: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() && force_user.is_none() {
                user = Some(b.to_vec());
            } else {
                eprintln!("login: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let _ = preserve;
        let _ = host;

        let euid = unsafe { libc::geteuid() };
        let name: Vec<u8> = if let Some(f) = force_user {
            if euid != 0 {
                eprintln!("login: -f only for root");
                return Ok(1);
            }
            f
        } else if let Some(u) = user {
            u
        } else {
            let stdout = io::stdout();
            {
                let mut o = stdout.lock();
                o.write_all(b"login: ")?;
                o.flush()?;
            }
            let mut line = Vec::new();
            let mut b = [0u8; 1];
            loop {
                match io::stdin().lock().read(&mut b) {
                    Ok(0) => break,
                    Ok(_) => {
                        if b[0] == b'\n' {
                            break;
                        }
                        line.push(b[0]);
                    }
                    Err(_) => break,
                }
            }

            while line.last() == Some(&b' ')
                || line.last() == Some(&b'\t')
                || line.last() == Some(&b'\r')
            {
                line.pop();
            }
            line
        };
        if name.is_empty() || name.contains(&b':') || name.contains(&b' ') {
            eprintln!("login: invalid login name");
            return Ok(1);
        }
        let ent = match lookup_user(&name) {
            Some(e) => e,
            None => {
                let _ = read_secret(b"Password: ", false);
                eprintln!("login: unknown user '{}'", String::from_utf8_lossy(&name));
                return Ok(1);
            }
        };

        if ent.uid != 0 {
            let nologin = acct_path("BB_NOLOGIN", "/etc/nologin");
            if nologin.exists() {
                match fs::read(&nologin) {
                    Ok(d) => {
                        let stdout = io::stdout();
                        let mut o = stdout.lock();
                        o.write_all(&d)?;
                        if !d.ends_with(b"\n") {
                            o.write_all(b"\n")?;
                        }
                    }
                    Err(_) => {
                        eprintln!("login: system is not available");
                    }
                }
                return Ok(1);
            }
        }
        let hash = shadow_hash(&name).unwrap_or_default();
        let pw = match read_secret(b"Password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("login: input error");
                return Ok(1);
            }
        };
        match verify_password(&hash, &pw) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("login: incorrect password");
                return Ok(1);
            }
            Err(()) => return no_crypt("login"),
        }

        if !ent.home.is_empty() {
            if let Some(h) = cstr(&ent.home) {
                if unsafe { libc::chdir(h.as_ptr()) } != 0 {
                    eprintln!(
                        "login: cannot chdir to '{}'",
                        String::from_utf8_lossy(&ent.home)
                    );
                }
            }
        }
        become_and_exec(&ent, None, true, None)
    }
}
