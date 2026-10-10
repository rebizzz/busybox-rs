use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct SetarchApplet;
impl Applet for SetarchApplet {
    fn name(&self) -> &'static str {
        "setarch"
    }
    fn description(&self) -> &'static str {
        "Change reported architecture and run a program"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut arch: Option<&[u8]> = None;
        let mut at: Option<usize> = None;
        let mut no_randomize = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--32" {
                arch = Some(b"linux32");
            } else if b == b"--64" {
                arch = Some(b"linux64");
            } else if b == b"-R" {
                no_randomize = true;
            } else if b.first() == Some(&b'-') {
                eprintln!("setarch: unknown option");
                return Ok(1);
            } else if arch.is_none() {
                arch = Some(b);
            } else {
                at = Some(i);
                break;
            }
            i += 1;
        }
        let arch = match arch {
            Some(a) => a,
            None => {
                eprintln!("usage: setarch ARCH [PROG...]");
                return Ok(1);
            }
        };

        let mut persona: libc::c_ulong = if arch.eq_ignore_ascii_case(b"i386")
            || arch.eq_ignore_ascii_case(b"i486")
            || arch.eq_ignore_ascii_case(b"i586")
            || arch.eq_ignore_ascii_case(b"i686")
            || arch.eq_ignore_ascii_case(b"linux32")
        {
            PER_LINUX32
        } else if arch.eq_ignore_ascii_case(b"x86_64")
            || arch.eq_ignore_ascii_case(b"amd64")
            || arch.eq_ignore_ascii_case(b"linux64")
        {
            PER_LINUX
        } else {
            eprintln!(
                "setarch: unknown architecture '{}'",
                String::from_utf8_lossy(arch)
            );
            return Ok(1);
        };
        if no_randomize {
            persona |= 0x0040000;
        }
        let prog = at.map(|p| args[p..].to_vec());
        Ok(do_personality(persona, prog))
    }
}
