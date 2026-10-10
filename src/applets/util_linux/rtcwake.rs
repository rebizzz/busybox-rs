use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct RtcwakeApplet;
impl Applet for RtcwakeApplet {
    fn name(&self) -> &'static str {
        "rtcwake"
    }
    fn description(&self) -> &'static str {
        "Set RTC wakeup alarm and suspend"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = [0u8; 16];
        let mut mode_len = 0usize;
        let mut seconds: u64 = 0;
        let mut have_mode = false;
        let mut have_time = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-m" || b.starts_with(b"--mode") {
                let v: &[u8];
                if b == b"-m" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: -m needs a mode");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                } else if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    v = &b[eq + 1..];
                } else {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: --mode needs a value");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                }
                let n = v.len().min(mode.len());
                mode[..n].copy_from_slice(&v[..n]);
                mode_len = n;
                have_mode = true;
            } else if b == b"-s" || b.starts_with(b"--seconds") {
                let v: &[u8];
                if b == b"-s" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: -s needs seconds");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                } else if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    v = &b[eq + 1..];
                } else {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: --seconds needs a value");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                }
                match std::str::from_utf8(v)
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    Some(s) => {
                        seconds = s;
                        have_time = true;
                    }
                    None => {
                        eprintln!("rtcwake: invalid seconds");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("rtcwake: unknown option");
                return Ok(1);
            }
            i += 1;
        }
        if !have_mode || !have_time {
            eprintln!("usage: rtcwake -m MODE -s SECONDS");
            return Ok(1);
        }

        let mut rtc: [i32; 9] = [0; 9];
        let mut fd = -1;
        for dev in ["/dev/rtc0", "/dev/rtc"] {
            use std::ffi::CString;
            if let Ok(c) = CString::new(dev) {
                let f = unsafe { libc::open(c.as_ptr(), libc::O_RDWR) };
                if f >= 0 {
                    fd = f;
                    break;
                }
            }
        }
        if fd < 0 {
            eprintln!("rtcwake: cannot open RTC device: no such device");
            return Ok(1);
        }
        let ok = unsafe { libc::ioctl(fd, RTC_RD_TIME, rtc.as_mut_ptr()) } == 0;
        unsafe { libc::close(fd) };
        if !ok {
            eprintln!("rtcwake: cannot read RTC time");
            return Ok(1);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) } as u64;
        let wake = now.saturating_add(seconds);
        let mut s = Vec::with_capacity(24);
        push_u64(&mut s, wake);
        match std::fs::write("/sys/class/rtc/rtc0/wakealarm", &s) {
            Ok(()) => {
                eprintln!(
                    "rtcwake: wakeup set {}s from now (mode {})",
                    seconds,
                    String::from_utf8_lossy(&mode[..mode_len])
                );
                Ok(0)
            }
            Err(e) => {
                eprintln!("rtcwake: cannot set wakealarm: {}", e);
                Ok(1)
            }
        }
    }
}
