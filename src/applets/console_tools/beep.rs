use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct BeepApplet;

impl Applet for BeepApplet {
    fn name(&self) -> &'static str {
        "beep"
    }

    fn description(&self) -> &'static str {
        "Beep the console speaker"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut freq: u32 = 440;
        let mut length_ms: u32 = 200;
        let mut reps: u32 = 1;
        let mut delay_ms: u32 = 100;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-f" && i + 1 < args.len() {
                i += 1;
                freq = parse_u32(args[i].as_bytes()).unwrap_or(440);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                length_ms = parse_u32(args[i].as_bytes()).unwrap_or(200);
            } else if b == b"-r" && i + 1 < args.len() {
                i += 1;
                reps = parse_u32(args[i].as_bytes()).unwrap_or(1);
            } else if b == b"-d" && i + 1 < args.len() {
                i += 1;
                delay_ms = parse_u32(args[i].as_bytes()).unwrap_or(100);
            }
            i += 1;
        }

        if let Ok(console) = open_console() {
            let period = freq
                .checked_div(1)
                .and_then(|_| 1193180u32.checked_div(freq))
                .unwrap_or(0);
            for r in 0..reps {
                unsafe {
                    libc::ioctl(console.as_raw_fd(), KIOCSOUND, period as libc::c_ulong);
                    libc::usleep((length_ms * 1000) as libc::useconds_t);
                    libc::ioctl(console.as_raw_fd(), KIOCSOUND, 0 as libc::c_ulong);
                }
                if r + 1 < reps {
                    unsafe {
                        libc::usleep((delay_ms * 1000) as libc::useconds_t);
                    }
                }
            }
        } else {
            let mut out = io::stdout().lock();
            for _ in 0..reps {
                let _ = out.write_all(b"\x07");
                let _ = out.flush();
            }
        }
        Ok(0)
    }
}
