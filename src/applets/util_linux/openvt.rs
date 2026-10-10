use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::process::{Command, Stdio};
use std::env;

pub struct OpenvtApplet;
impl Applet for OpenvtApplet {
    fn name(&self) -> &'static str {
        "openvt"
    }
    fn description(&self) -> &'static str {
        "Start a program on a new virtual terminal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut vtno: Option<i32> = None;
        let mut switch = false;
        let mut wait = false;
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" && i + 1 < args.len() {
                vtno = args[i + 1].to_string_lossy().parse().ok();
                i += 2;
                continue;
            } else if b == b"-s" {
                switch = true;
            } else if b == b"-w" {
                wait = true;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        let c_console = CString::new("/dev/console").unwrap();
        let console_fd = unsafe { libc::open(c_console.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
        if console_fd < 0 {
            eprintln!("openvt: can't open /dev/console");
            return Ok(1);
        }

        let vt_num = match vtno {
            Some(v) => v,
            None => {
                let mut free_vt = 0i32;
                if unsafe { libc::ioctl(console_fd, VT_OPENQRY, &mut free_vt) } != 0 || free_vt <= 0
                {
                    unsafe { libc::close(console_fd) };
                    eprintln!("openvt: can't find free VT");
                    return Ok(1);
                }
                free_vt
            }
        };

        if switch {
            unsafe {
                libc::ioctl(console_fd, VT_ACTIVATE, vt_num);
                libc::ioctl(console_fd, VT_WAITACTIVE, vt_num);
            }
        }
        unsafe { libc::close(console_fd) };

        let vt_dev = format!("/dev/tty{}", vt_num);
        let vt_c = CString::new(vt_dev).unwrap();
        let vt_fd = unsafe { libc::open(vt_c.as_ptr(), libc::O_RDWR) };

        let cmd_parts = if cmd_idx < args.len() {
            &args[cmd_idx..]
        } else {
            &[]
        };

        let prog = if !cmd_parts.is_empty() {
            cmd_parts[0].to_string_lossy().to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
        };

        let mut cmd = Command::new(&prog);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }

        if vt_fd >= 0 {
            let stdio_fd = unsafe { Stdio::from_raw_fd(vt_fd) };
            cmd.stdin(stdio_fd);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                if wait {
                    let st = child.wait().map(|s| s.code().unwrap_or(0)).unwrap_or(1);
                    Ok(st)
                } else {
                    Ok(0)
                }
            }
            Err(e) => {
                eprintln!("openvt: {}: {}", prog, e);
                Ok(1)
            }
        }
    }
}
