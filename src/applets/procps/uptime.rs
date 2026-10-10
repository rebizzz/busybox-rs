use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct UptimeApplet;
impl Applet for UptimeApplet {
    fn name(&self) -> &'static str {
        "uptime"
    }
    fn description(&self) -> &'static str {
        "Show how long the system has been running"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let since = args.iter().any(|a| a.as_bytes() == b"-s");
        let mut ub = [0u8; 64];
        let n = read_small("/proc/uptime", &mut ub);
        if n == 0 {
            eprintln!("uptime: cannot read /proc/uptime");
            return Ok(1);
        }

        let mut secs: u64 = 0;
        let mut frac: u64 = 0;
        let mut i = 0;
        while i < n && ub[i].is_ascii_digit() {
            secs = secs
                .saturating_mul(10)
                .saturating_add((ub[i] - b'0') as u64);
            i += 1;
        }
        if i < n && ub[i] == b'.' {
            i += 1;
            let mut mul = 100;
            while i < n && ub[i].is_ascii_digit() && mul > 0 {
                frac += ((ub[i] - b'0') as u64) * mul;
                mul /= 10;
                i += 1;
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(128);
        if since {
            let now = unsafe { libc::time(std::ptr::null_mut()) } as i64 - secs as i64;
            let mut tm: libc::tm = unsafe { std::mem::zeroed() };
            unsafe { libc::localtime_r(&now, &mut tm) };
            fn p2(out: &mut Vec<u8>, v: i32) {
                out.push(b'0' + (v / 10) as u8);
                out.push(b'0' + (v % 10) as u8);
            }
            push_u64(&mut line, (tm.tm_year + 1900) as u64);
            line.push(b'-');
            p2(&mut line, tm.tm_mon + 1);
            line.push(b'-');
            p2(&mut line, tm.tm_mday);
            line.push(b' ');
            p2(&mut line, tm.tm_hour);
            line.push(b':');
            p2(&mut line, tm.tm_min);
            line.push(b':');
            p2(&mut line, tm.tm_sec);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&now, &mut tm) };
        fn p2b(out: &mut Vec<u8>, v: i32) {
            out.push(b'0' + ((v / 10) % 10) as u8);
            out.push(b'0' + (v % 10) as u8);
        }
        line.push(b' ');
        p2b(&mut line, tm.tm_hour);
        line.push(b':');
        p2b(&mut line, tm.tm_min);
        line.push(b':');
        p2b(&mut line, tm.tm_sec);
        line.extend_from_slice(b" up ");
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            push_u64(&mut line, days);
            line.extend_from_slice(if days == 1 { b" day, " } else { b" days, " });
        }
        if days > 0 || hours > 0 {
            push_u64(&mut line, hours);
            line.push(b':');
            if mins < 10 {
                line.push(b'0');
            }
            push_u64(&mut line, mins);
        } else {
            push_u64(&mut line, mins);
            line.extend_from_slice(b" min");
        }

        let mut lb = [0u8; 64];
        let ln = read_small("/proc/loadavg", &mut lb);
        if ln > 0 {
            line.extend_from_slice(b",  load average: ");
            let mut fields = 0;
            for &c in &lb[..ln] {
                if c == b'\n' {
                    break;
                }
                line.push(c);
                if c == b' ' {
                    fields += 1;
                    if fields == 3 {
                        line.pop();
                        break;
                    }
                }
            }
        }
        let _ = frac;
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
