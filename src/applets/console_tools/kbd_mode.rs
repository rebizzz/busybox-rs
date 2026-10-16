use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct KbdModeApplet;

impl Applet for KbdModeApplet {
    fn name(&self) -> &'static str {
        "kbd_mode"
    }

    fn description(&self) -> &'static str {
        "Report or set keyboard mode"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("kbd_mode: {}", e);
                return Ok(1);
            }
        };

        if args.len() <= 1 {
            let mut mode: libc::c_int = 0;
            let ret = unsafe {
                libc::ioctl(
                    console.as_raw_fd(),
                    KDGKBMODE,
                    &mut mode as *mut libc::c_int as *mut libc::c_void,
                )
            };
            if ret < 0 {
                eprintln!("kbd_mode: KDGKBMODE failed: {}", io::Error::last_os_error());
                return Ok(1);
            }
            let mode_str = match mode {
                0 => "raw (SCANCODE)",
                1 => "mediumraw (KEYCODE)",
                2 => "default (ASCII)",
                3 => "Unicode (UTF-8)",
                4 => "off",
                _ => "unknown",
            };
            println!("The keyboard is in {} mode", mode_str);
            return Ok(0);
        }

        let arg = args[1].as_bytes();
        let new_mode = match arg {
            b"-s" => 0,
            b"-k" => 1,
            b"-a" => 2,
            b"-u" => 3,
            _ => {
                eprintln!("Usage: kbd_mode [-a|-k|-s|-u]");
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(console.as_raw_fd(), KDSKBMODE, new_mode as libc::c_ulong) };
        if ret < 0 {
            eprintln!("kbd_mode: KDSKBMODE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
