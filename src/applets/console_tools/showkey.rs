use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct ShowkeyApplet;

impl Applet for ShowkeyApplet {
    fn name(&self) -> &'static str {
        "showkey"
    }

    fn description(&self) -> &'static str {
        "Display scancodes or keycodes sent by the keyboard"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode_k = true;
        let mut timeout_sec = 10;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                mode_k = false;
            } else if b == b"-k" {
                mode_k = true;
            } else if b == b"-a" {
                mode_k = false;
            } else if b.starts_with(b"-t") {
                let rest = if b.len() > 2 {
                    &b[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if let Some(t) = parse_u32(rest) {
                    timeout_sec = t as i32;
                }
            }
            i += 1;
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("showkey: {}", e);
                return Ok(1);
            }
        };

        let mut old_mode: libc::c_int = 0;
        unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                KDGKBMODE,
                &mut old_mode as *mut libc::c_int as *mut libc::c_void,
            );
        }

        let new_mode = if mode_k { 2 } else { 1 };
        unsafe {
            libc::ioctl(console.as_raw_fd(), KDSKBMODE, new_mode as libc::c_ulong);
        }

        println!(
            "Press any keys (program exits {}s after last keypress)...",
            timeout_sec
        );
        let mut buf = [0u8; 64];
        let fd = console.as_raw_fd();
        loop {
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_sec * 1000) };
            if ret <= 0 {
                break;
            }
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for &byte in &buf[..n as usize] {
                let keycode = (byte & 0x7F) as u32;
                let release = (byte & 0x80) != 0;
                if mode_k {
                    println!(
                        "keycode {} {}",
                        keycode,
                        if release { "release" } else { "press" }
                    );
                } else {
                    println!("0x{:02x}", byte);
                }
            }
        }

        unsafe {
            libc::ioctl(console.as_raw_fd(), KDSKBMODE, old_mode as libc::c_ulong);
        }
        Ok(0)
    }
}
