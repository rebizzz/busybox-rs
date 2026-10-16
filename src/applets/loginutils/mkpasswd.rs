use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct MkpasswdApplet;
impl Applet for MkpasswdApplet {
    fn name(&self) -> &'static str {
        "mkpasswd"
    }
    fn description(&self) -> &'static str {
        "Hash a password with crypt(3) (unavailable: fails loudly)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "mkpasswd",
                    "[-P FD] [-m TYPE] [-S SALT] [PASS] [SALT]",
                    "Print a crypted password (needs crypt)",
                );
            } else if b == b"-P" || b == b"-m" || b == b"-S" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mkpasswd: option requires an argument");
                    return Ok(1);
                }
            } else if b.starts_with(b"-") {
                eprintln!("mkpasswd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        no_crypt("mkpasswd")
    }
}
