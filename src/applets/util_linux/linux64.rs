use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct Linux64Applet;
impl Applet for Linux64Applet {
    fn name(&self) -> &'static str {
        "linux64"
    }
    fn description(&self) -> &'static str {
        "Run a program with 64-bit personality"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut cmd: Vec<OsString> = Vec::new();
        for a in args {
            if cmd.is_empty() && a.as_bytes() == b"--64" {
                continue;
            } else if cmd.is_empty() && a.as_bytes().first() == Some(&b'-') {
                eprintln!("linux64: unknown option");
                return Ok(1);
            } else {
                cmd.push(a.clone());
            }
        }
        let prog = if cmd.is_empty() { None } else { Some(cmd) };
        Ok(do_personality(PER_LINUX, prog))
    }
}
