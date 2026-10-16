use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct TtyApplet;

impl Applet for TtyApplet {
    fn name(&self) -> &'static str {
        "tty"
    }
    fn description(&self) -> &'static str {
        "Print file name of terminal on stdin"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut silent = false;
        for arg in args {
            if arg.as_bytes() == b"-s" || arg.as_bytes() == b"--silent" {
                silent = true;
            }
        }

        let isatty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
        if !isatty {
            if !silent {
                println!("not a tty");
            }
            return Ok(1);
        }

        if !silent {
            let name_ptr = unsafe { libc::ttyname(libc::STDIN_FILENO) };
            if !name_ptr.is_null() {
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
                println!("{}", name.to_string_lossy());
            } else {
                println!("not a tty");
                return Ok(1);
            }
        }

        Ok(0)
    }
}

