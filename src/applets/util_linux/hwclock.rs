use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct HwclockApplet;
impl Applet for HwclockApplet {
    fn name(&self) -> &'static str {
        "hwclock"
    }
    fn description(&self) -> &'static str {
        "Read or set the hardware clock"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut utc = false;
        let mut hctosys = false;
        let mut systohc = false;
        for a in args {
            let b = a.as_bytes();
            if b == b"-r" || b == b"--show" {
            } else if b == b"-u" || b == b"--utc" {
                utc = true;
            } else if b == b"-l" || b == b"--localtime" {
                utc = false;
            } else if b == b"--hctosys" || b == b"-s" {
                hctosys = true;
            } else if b == b"--systohc" || b == b"-w" {
                systohc = true;
            } else {
                eprintln!("hwclock: unknown option");
                return Ok(1);
            }
        }
        if hctosys || systohc {
            eprintln!("hwclock: cannot access hardware clock: no RTC device");
            return Ok(1);
        }

        let mut rtc: [i32; 9] = [0; 9];
        let mut got_rtc = false;
        for dev in ["/dev/rtc0", "/dev/rtc"] {
            use std::ffi::CString;
            if let Ok(c) = CString::new(dev) {
                let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY) };
                if fd >= 0 {
                    if unsafe { libc::ioctl(fd, RTC_RD_TIME, rtc.as_mut_ptr()) } == 0 {
                        got_rtc = true;
                    }
                    unsafe { libc::close(fd) };
                    if got_rtc {
                        break;
                    }
                }
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if got_rtc {
            let mut line = Vec::with_capacity(48);
            push_u64(&mut line, (rtc[5] + 1900) as u64);
            line.push(b'-');
            let m = (rtc[4] + 1) as u8;
            line.push(b'0' + m / 10);
            line.push(b'0' + m % 10);
            line.push(b'-');
            line.push(b'0' + (rtc[3] / 10) as u8);
            line.push(b'0' + (rtc[3] % 10) as u8);
            line.push(b' ');
            for v in [rtc[2], rtc[1], rtc[0]] {
                line.push(b'0' + (v / 10) as u8);
                line.push(b'0' + (v % 10) as u8);
                line.push(b':');
            }
            line.pop();
            if utc {
                line.extend_from_slice(b" UTC");
            }
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if utc {
            unsafe { libc::gmtime_r(&now, &mut tm) };
        } else {
            unsafe { libc::localtime_r(&now, &mut tm) };
        }
        eprintln!("hwclock: no RTC device, showing system time");
        let mut line = Vec::with_capacity(48);
        push_u64(&mut line, (tm.tm_year + 1900) as u64);
        line.push(b'-');
        line.push(b'0' + ((tm.tm_mon + 1) / 10) as u8);
        line.push(b'0' + ((tm.tm_mon + 1) % 10) as u8);
        line.push(b'-');
        line.push(b'0' + (tm.tm_mday / 10) as u8);
        line.push(b'0' + (tm.tm_mday % 10) as u8);
        line.push(b' ');
        for v in [tm.tm_hour, tm.tm_min, tm.tm_sec] {
            line.push(b'0' + (v / 10) as u8);
            line.push(b'0' + (v % 10) as u8);
            line.push(b':');
        }
        line.pop();
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
