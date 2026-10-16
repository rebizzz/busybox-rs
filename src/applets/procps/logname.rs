use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct LognameApplet;
impl Applet for LognameApplet {
    fn name(&self) -> &'static str {
        "logname"
    }
    fn description(&self) -> &'static str {
        "Print the name of the current user"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("logname: too many arguments");
            return Ok(1);
        }
        unsafe {
            let p = libc::getlogin();
            if !p.is_null() {
                let name = cstr_field(p);
                if !name.is_empty() {
                    let stdout = std::io::stdout();
                    let mut out = stdout.lock();
                    out.write_all(&name)?;
                    out.write_all(b"\n")?;
                    out.flush()?;
                    return Ok(0);
                }
            }
        }

        if let Some(user) = crate::core::platform::get_current_username() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(user.as_bytes())?;
            out.write_all(b"\n")?;
            out.flush()?;
            Ok(0)
        } else {
            eprintln!("logname: no login name");
            Ok(1)
        }
    }
}
