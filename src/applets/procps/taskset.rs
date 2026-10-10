use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct TasksetApplet;
impl Applet for TasksetApplet {
    fn name(&self) -> &'static str {
        "taskset"
    }
    fn description(&self) -> &'static str {
        "Get or set CPU affinity"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_pid_only = false;
        let mut rest: Vec<OsString> = Vec::new();
        for a in args {
            if a.as_bytes() == b"-p" {
                show_pid_only = true;
            } else {
                rest.push(a.clone());
            }
        }
        let show = |pid: i32| -> Result<i32> {
            let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
            let sz = std::mem::size_of::<libc::cpu_set_t>();
            if unsafe { libc::sched_getaffinity(pid, sz, &mut set) } != 0 {
                eprintln!("taskset: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let words: &[u64] =
                unsafe { std::slice::from_raw_parts((&raw const set) as *const u64, sz / 8) };
            let mut mask: u64 = 0;
            for (k, w) in words.iter().enumerate().take(1) {
                let _ = k;
                mask = *w;
            }
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(48);
            line.extend_from_slice(b"pid ");
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b"'s current affinity mask: ");
            let mut hex = [0u8; 16];
            let mut hn = 0;
            let mut v = mask;
            if v == 0 {
                hex[0] = b'0';
                hn = 1;
            } else {
                let mut rev = [0u8; 16];
                let mut rn = 0;
                while v > 0 {
                    let d = (v & 0xf) as u8;
                    rev[rn] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                    v >>= 4;
                    rn += 1;
                }
                while rn > 0 {
                    rn -= 1;
                    hex[hn] = rev[rn];
                    hn += 1;
                }
            }
            line.extend_from_slice(&hex[..hn]);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            Ok(0)
        };
        let parse_mask = |b: &[u8]| -> Option<u64> {
            let h = if b.starts_with(b"0x") || b.starts_with(b"0X") {
                &b[2..]
            } else {
                b
            };

            let g = match h.iter().rposition(|&c| c == b',') {
                Some(p) => &h[p + 1..],
                None => h,
            };
            if g.is_empty() || !g.iter().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            u64::from_str_radix(std::str::from_utf8(g).ok()?, 16).ok()
        };
        match rest.len() {
            0 => {
                eprintln!("usage: taskset [-p] [mask] pid|cmd...");
                Ok(1)
            }
            1 => {
                if show_pid_only {
                    match std::str::from_utf8(rest[0].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        Some(pid) => show(pid),
                        None => {
                            eprintln!("taskset: invalid pid");
                            Ok(1)
                        }
                    }
                } else {
                    if let Some(pid) = std::str::from_utf8(rest[0].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        show(pid)
                    } else {
                        eprintln!("usage: taskset [-p] [mask] pid|cmd...");
                        Ok(1)
                    }
                }
            }
            _ => {
                let mask = match parse_mask(rest[0].as_bytes()) {
                    Some(m) => m,
                    None => {
                        eprintln!("taskset: invalid mask");
                        return Ok(1);
                    }
                };

                if rest.len() == 2
                    && std::str::from_utf8(rest[1].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                        .is_some()
                {
                    let pid: i32 = std::str::from_utf8(rest[1].as_bytes())
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let sz = std::mem::size_of::<libc::cpu_set_t>();
                    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
                    unsafe {
                        let words =
                            std::slice::from_raw_parts_mut((&raw mut set) as *mut u64, sz / 8);
                        for w in words.iter_mut() {
                            *w = 0;
                        }
                        words[0] = mask;
                        if libc::sched_setaffinity(pid, sz, &set) != 0 {
                            eprintln!("taskset: {}", std::io::Error::last_os_error());
                            return Ok(1);
                        }
                    }
                    return show(pid);
                }

                let sz = std::mem::size_of::<libc::cpu_set_t>();
                let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
                unsafe {
                    let words = std::slice::from_raw_parts_mut((&raw mut set) as *mut u64, sz / 8);
                    for w in words.iter_mut() {
                        *w = 0;
                    }
                    words[0] = mask;
                    if libc::sched_setaffinity(0, sz, &set) != 0 {
                        eprintln!("taskset: {}", std::io::Error::last_os_error());
                        return Ok(1);
                    }
                }
                let cmd: Vec<OsString> = rest[1..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
        }
    }
}
