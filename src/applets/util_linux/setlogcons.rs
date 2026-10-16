use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::io::{self};

pub struct SetlogconsApplet;
impl Applet for SetlogconsApplet {
    fn name(&self) -> &'static str {
        "setlogcons"
    }
    fn description(&self) -> &'static str {
        "Pin kernel output to VT console N"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let vtno: u8 = if !args.is_empty() {
            args[0].to_string_lossy().parse().unwrap_or(0)
        } else {
            0
        };

        #[repr(C)]
        struct TiocLinuxArg {
            fn_: u8,
            subarg: u8,
        }

        let arg = TiocLinuxArg {
            fn_: 11,
            subarg: vtno,
        };

        let tty_path = format!("/dev/tty{}", vtno);
        let c_path = CString::new(tty_path).unwrap();
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
        let target_fd = if fd >= 0 { fd } else { libc::STDIN_FILENO };

        let res = unsafe { libc::ioctl(target_fd, TIOCLINUX, &arg) };
        if fd >= 0 {
            unsafe { libc::close(fd) };
        }

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("setlogcons: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}
