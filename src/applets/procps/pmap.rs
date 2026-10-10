use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct PmapApplet;
impl Applet for PmapApplet {
    fn name(&self) -> &'static str {
        "pmap"
    }
    fn description(&self) -> &'static str {
        "Report memory map of processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ext = false;
        let mut pids: Vec<u32> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-x" {
                ext = true;
            } else if b == b"-q" {
            } else if b.first() == Some(&b'-') {
                eprintln!("pmap: unknown option");
                return Ok(1);
            } else {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => pids.push(p),
                    None => {
                        eprintln!("pmap: invalid pid");
                        return Ok(1);
                    }
                }
            }
        }
        if pids.is_empty() {
            eprintln!("usage: pmap [-x] pid...");
            return Ok(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for pid in pids {
            let mut path = [0u8; 48];
            let n = proc_path(pid, b"/maps", &mut path);
            let ps = match std::str::from_utf8(&path[..n]) {
                Ok(s) => s.to_string(),
                Err(_) => continue,
            };
            let data = match std::fs::read(&ps) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("pmap: {}: {}", pid, e);
                    rc = 1;
                    continue;
                }
            };
            let mut line = Vec::with_capacity(64);
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b": maps\n");
            out.write_all(&line)?;
            out.write_all(&data)?;

            let mut total: u64 = 0;
            for ln in data.split(|&c| c == b'\n') {
                if let Some(dash) = ln.iter().position(|&c| c == b'-') {
                    let mut sp = dash;
                    while sp < ln.len() && ln[sp] != b' ' {
                        sp += 1;
                    }
                    let lo =
                        u64::from_str_radix(std::str::from_utf8(&ln[..dash]).unwrap_or(""), 16)
                            .unwrap_or(0);
                    let hi = u64::from_str_radix(
                        std::str::from_utf8(&ln[dash + 1..sp]).unwrap_or(""),
                        16,
                    )
                    .unwrap_or(0);
                    total = total.saturating_add(hi.saturating_sub(lo));
                }
            }
            if ext {
                let mut spath = [0u8; 48];
                let sn = proc_path(pid, b"/smaps", &mut spath);
                let mut rss: u64 = 0;
                let mut pss: u64 = 0;
                if let Ok(ss) = std::str::from_utf8(&spath[..sn]) {
                    let mut sb = [0u8; 65536];
                    let rn = read_small(ss, &mut sb);
                    let mut k = 0;
                    while k < rn {
                        let mut e = k;
                        while e < rn && sb[e] != b'\n' {
                            e += 1;
                        }
                        let l = &sb[k..e];
                        if l.starts_with(b"Rss:") || l.starts_with(b"Pss:") {
                            let mut j = 4;
                            while j < l.len() && !l[j].is_ascii_digit() {
                                j += 1;
                            }
                            let mut v: u64 = 0;
                            while j < l.len() && l[j].is_ascii_digit() {
                                v = v * 10 + (l[j] - b'0') as u64;
                                j += 1;
                            }
                            if l.starts_with(b"Rss:") {
                                rss += v;
                            } else {
                                pss += v;
                            }
                        }
                        k = e + 1;
                    }
                }
                let mut tl = Vec::with_capacity(64);
                tl.extend_from_slice(b"total ");
                push_u64(&mut tl, total / 1024);
                tl.extend_from_slice(b"K rss ");
                push_u64(&mut tl, rss);
                tl.extend_from_slice(b"K pss ");
                push_u64(&mut tl, pss);
                tl.extend_from_slice(b"K\n");
                out.write_all(&tl)?;
            } else {
                let mut tl = Vec::with_capacity(32);
                tl.extend_from_slice(b"total ");
                push_u64(&mut tl, total / 1024);
                tl.extend_from_slice(b"K\n");
                out.write_all(&tl)?;
            }
        }
        out.flush()?;
        Ok(rc)
    }
}
