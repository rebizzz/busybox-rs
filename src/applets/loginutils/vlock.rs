use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct VlockApplet;
impl Applet for VlockApplet {
    fn name(&self) -> &'static str {
        "vlock"
    }
    fn description(&self) -> &'static str {
        "Lock the terminal until the session password is entered"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("vlock", "[-a]", "Lock the terminal (current tty subset)");
            } else if a.as_bytes() == b"-a" {
            } else {
                eprintln!(
                    "vlock: invalid option '{}'",
                    String::from_utf8_lossy(a.as_bytes())
                );
                return Ok(1);
            }
        }
        let p1 = match read_secret(b"Password: ", false) {
            Some(p) if !p.is_empty() => p,
            _ => {
                eprintln!("vlock: empty password, not locking");
                return Ok(1);
            }
        };
        let p2 = match read_secret(b"Password (again): ", false) {
            Some(p) => p,
            None => {
                eprintln!("vlock: input error");
                return Ok(1);
            }
        };
        if p1 != p2 {
            eprintln!("vlock: passwords do not match");
            return Ok(1);
        }
        eprintln!("vlock: terminal locked, enter password to unlock");
        loop {
            match read_secret(b"Password: ", false) {
                Some(p) if p == p1 => break,
                Some(_) => eprintln!("vlock: incorrect password"),
                None => {
                    eprintln!("vlock: input closed while locked");
                    return Ok(1);
                }
            }
        }
        Ok(0)
    }
}
