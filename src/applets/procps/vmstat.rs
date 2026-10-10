use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct VmstatApplet;
impl Applet for VmstatApplet {
    fn name(&self) -> &'static str {
        "vmstat"
    }
    fn description(&self) -> &'static str {
        "Report virtual memory statistics"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.iter().any(|a| a.as_bytes() == b"-s") {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for (k, key) in [
                b"MemTotal".as_slice(),
                b"MemFree".as_slice(),
                b"Buffers".as_slice(),
                b"Cached".as_slice(),
                b"SwapTotal".as_slice(),
                b"SwapFree".as_slice(),
            ]
            .iter()
            .enumerate()
            {
                let _ = k;
                let mut line = Vec::with_capacity(48);
                push_u64(&mut line, meminfo_kb(key));
                line.extend_from_slice(b" kB ");
                line.extend_from_slice(key);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            return Ok(0);
        }
        let mut delay_ms = 0;
        let mut count: u64 = 1;
        for (pos, a) in args.iter().enumerate() {
            if a.as_bytes().first() == Some(&b'-') {
                eprintln!("vmstat: unknown option");
                return Ok(1);
            }
            if pos == 0 {
                delay_ms = parse_delay(a.as_bytes());
            } else if pos == 1 {
                count = std::str::from_utf8(a.as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(
            b"procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----\n",
        )?;
        out.write_all(
            b" r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id\n",
        )?;
        if count == 0 {
            count = u64::MAX / 2;
        }
        for it in 0..count {
            vmstat_line(&mut out)?;
            out.flush()?;
            if it + 1 < count && delay_ms > 0 {
                let ts = libc::timespec {
                    tv_sec: (delay_ms / 1000) as libc::time_t,
                    tv_nsec: ((delay_ms % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
        }
        Ok(0)
    }
}
