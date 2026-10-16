#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct MesgApplet;
impl Applet for MesgApplet {
    fn name(&self) -> &'static str {
        "mesg"
    }
    fn description(&self) -> &'static str {
        "Control write access to your terminal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out(
                    "mesg",
                    "[y|n]",
                    "Allow or forbid write access to your terminal",
                );
            }
        }
        if args.len() > 1 {
            eprintln!("Usage: mesg [y|n]");
            return Ok(1);
        }

        let fd: libc::c_int = unsafe {
            if libc::isatty(0) == 1 {
                0
            } else {
                let c = cstr(b"/dev/tty").unwrap();
                let f = libc::open(c.as_ptr(), libc::O_RDONLY);
                if f < 0 {
                    eprintln!("mesg: no tty: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                f
            }
        };
        let close_fd = fd != 0;
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut st) } != 0 {
            eprintln!("mesg: fstat: {}", io::Error::last_os_error());
            if close_fd {
                unsafe { libc::close(fd) };
            }
            return Ok(1);
        }
        let rc = if args.is_empty() {
            let stdout = io::stdout();
            let mut o = stdout.lock();
            if st.st_mode & libc::S_IWGRP != 0 {
                o.write_all(b"y\n")?;
            } else {
                o.write_all(b"n\n")?;
            }
            Ok(0)
        } else {
            let want_y = match args[0].as_bytes() {
                b"y" => true,
                b"n" => false,
                other => {
                    eprintln!(
                        "mesg: invalid argument '{}'",
                        String::from_utf8_lossy(other)
                    );
                    if close_fd {
                        unsafe { libc::close(fd) };
                    }
                    return Ok(1);
                }
            };
            let mut mode = st.st_mode;
            if want_y {
                mode |= libc::S_IWGRP;
            } else {
                mode &= !libc::S_IWGRP;
            }

            if unsafe { libc::fchmod(fd, mode) } != 0 {
                eprintln!("mesg: cannot change mode: {}", io::Error::last_os_error());
                if close_fd {
                    unsafe { libc::close(fd) };
                }
                return Ok(1);
            }
            Ok(0)
        };
        if close_fd {
            unsafe { libc::close(fd) };
        }
        rc
    }
}
