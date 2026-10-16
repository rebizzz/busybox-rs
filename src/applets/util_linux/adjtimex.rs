use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct AdjtimexApplet;
impl Applet for AdjtimexApplet {
    fn name(&self) -> &'static str {
        "adjtimex"
    }
    fn description(&self) -> &'static str {
        "Print or set kernel time variables"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tx: libc::timex = unsafe { std::mem::zeroed() };
        tx.modes = 0;
        let r = unsafe { libc::adjtimex(&mut tx) };
        if r < 0 {
            eprintln!("adjtimex: {}", std::io::Error::last_os_error());
            return Ok(1);
        }

        let mut set_modes: u32 = 0;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            let next = || -> Option<i64> {
                if i + 1 >= args.len() {
                    return None;
                }
                std::str::from_utf8(args[i + 1].as_bytes())
                    .ok()?
                    .parse::<i64>()
                    .ok()
            };
            if b == b"--print" || b == b"-p" {
            } else if b == b"-o" {
                match next() {
                    Some(v) => {
                        tx.offset = v as libc::c_long;
                        set_modes |= 1;
                    }
                    None => {
                        eprintln!("adjtimex: -o needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else if b == b"-f" {
                match next() {
                    Some(v) => {
                        tx.freq = v as libc::c_long;
                        set_modes |= 2;
                    }
                    None => {
                        eprintln!("adjtimex: -f needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else if b == b"-t" {
                match next() {
                    Some(v) => {
                        tx.tick = v as libc::c_long;
                        set_modes |= 4;
                    }
                    None => {
                        eprintln!("adjtimex: -t needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else {
                eprintln!("adjtimex: unknown option");
                return Ok(1);
            }
            i += 1;
        }
        if set_modes != 0 {
            tx.modes = set_modes;
            if unsafe { libc::adjtimex(&mut tx) } < 0 {
                eprintln!("adjtimex: {}", std::io::Error::last_os_error());
                return Ok(1);
            }

            tx.modes = 0;
            if unsafe { libc::adjtimex(&mut tx) } < 0 {
                eprintln!("adjtimex: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(160);
        for (k, v) in [
            ("mode", tx.modes as i64),
            ("offset", tx.offset as i64),
            ("freq", tx.freq as i64),
            ("tick", tx.tick as i64),
            ("status", tx.status as i64),
        ] {
            line.extend_from_slice(k.as_bytes());
            line.extend_from_slice(b": ");
            if v < 0 {
                line.push(b'-');
                push_u64(&mut line, v.unsigned_abs());
            } else {
                push_u64(&mut line, v as u64);
            }
            line.push(b'\n');
        }
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

const RTC_RD_TIME: libc::c_ulong = 0x8024_7009;
