use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct UnameApplet;
impl Applet for UnameApplet {
    fn name(&self) -> &'static str {
        "uname"
    }
    fn description(&self) -> &'static str {
        "Print system information"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mask: u8 = 0;
        for a in args {
            let b = a.as_bytes();
            if b.first() == Some(&b'-') && b.len() > 1 && b[1] != b'-' {
                for &c in &b[1..] {
                    match c {
                        b's' => mask |= 1,
                        b'n' => mask |= 2,
                        b'r' => mask |= 4,
                        b'v' => mask |= 8,
                        b'm' => mask |= 16,
                        b'p' => mask |= 32,
                        b'i' => mask |= 64,
                        b'o' => mask |= 128,
                        b'a' => {
                            mask = 0xFF;
                            break;
                        }
                        _ => {}
                    }
                }
            } else if b == b"--all" {
                mask = 0xFF;
            }
        }
        if mask == 0 {
            mask = 1;
        }
        let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
        if unsafe { libc::uname(&mut uts) } != 0 {
            eprintln!("uname: cannot get system name");
            return Ok(1);
        }
        let fields: [Vec<u8>; 8] = [
            cstr_field(uts.sysname.as_ptr()),
            cstr_field(uts.nodename.as_ptr()),
            cstr_field(uts.release.as_ptr()),
            cstr_field(uts.version.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            b"GNU/Linux".to_vec(),
        ];
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut first = true;

        for (i, f) in fields.iter().enumerate() {
            if mask & (1 << i) != 0 {
                if !first {
                    out.write_all(b" ")?;
                }
                first = false;
                if mask == 0xFF && f == b"unknown" {
                    continue;
                }
                out.write_all(f)?;
            }
        }
        out.write_all(b"\n")?;
        out.flush()?;
        Ok(0)
    }
}
