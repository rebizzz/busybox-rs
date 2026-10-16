use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct SmemcapApplet;
impl Applet for SmemcapApplet {
    fn name(&self) -> &'static str {
        "smemcap"
    }
    fn description(&self) -> &'static str {
        "Report process memory from smaps (PSS)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() > 1 {
            eprintln!("usage: smemcap [FILE]");
            return Ok(1);
        }

        let mut rows: Vec<(u32, Vec<u8>, u64, u64)> = Vec::new();
        let dir = match std::fs::read_dir("/proc") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("smemcap: {}", e);
                return Ok(1);
            }
        };
        for e in dir.flatten() {
            let b = e.file_name();
            let bb = b.as_bytes();
            if bb.is_empty() || !bb.iter().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let pid: u32 = std::str::from_utf8(bb)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let mut path = [0u8; 64];
            let base = b"/proc/";
            path[..6].copy_from_slice(base);
            let mut len = 6;
            for &c in bb.iter().take(10) {
                if len + 7 < path.len() {
                    path[len] = c;
                    len += 1;
                }
            }
            path[len..len + 6].copy_from_slice(b"/smaps");
            let plen = len + 6;
            let ps = match std::str::from_utf8(&path[..plen]) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut sb = [0u8; 65536];
            let rn = read_small(ps, &mut sb);
            if rn == 0 {
                continue;
            }
            let mut rss: u64 = 0;
            let mut pss: u64 = 0;
            let mut k = 0;
            while k < rn {
                let mut en = k;
                while en < rn && sb[en] != b'\n' {
                    en += 1;
                }
                let l = &sb[k..en];
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
                k = en + 1;
            }

            let mut comm = Vec::new();
            let mut stp = [0u8; 64];
            stp[..6].copy_from_slice(base);
            let mut sl = 6;
            for &c in bb.iter().take(10) {
                if sl + 5 < stp.len() {
                    stp[sl] = c;
                    sl += 1;
                }
            }
            stp[sl..sl + 5].copy_from_slice(b"/stat");
            if let Ok(ss) = std::str::from_utf8(&stp[..sl + 5]) {
                let mut buf = [0u8; 512];
                let n = read_small(ss, &mut buf);
                if let Some(o) = buf[..n].iter().position(|&c| c == b'(') {
                    if let Some(cl) = buf[o..n].iter().position(|&c| c == b')') {
                        comm = buf[o + 1..o + cl].to_vec();
                    }
                }
            }
            rows.push((pid, comm, pss, rss));
        }
        rows.sort_by_key(|t| t.0);
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(b"PID COMMAND PSS(kB) RSS(kB)\n");
        let mut tp = 0u64;
        let mut tr = 0u64;
        for (pid, comm, pss, rss) in &rows {
            tp += *pss;
            tr += *rss;
            push_u64(&mut out, *pid as u64);
            out.push(b' ');
            out.extend_from_slice(comm);
            out.push(b' ');
            push_u64(&mut out, *pss);
            out.push(b' ');
            push_u64(&mut out, *rss);
            out.push(b'\n');
        }
        out.extend_from_slice(b"total ");
        push_u64(&mut out, tp);
        out.push(b' ');
        push_u64(&mut out, tr);
        out.push(b'\n');
        if args.is_empty() {
            let stdout = std::io::stdout();
            let mut o = stdout.lock();
            o.write_all(&out)?;
            o.flush()?;
        } else {
            let p = std::path::Path::new(args[0].as_os_str());
            if let Err(ee) = std::fs::write(p, &out) {
                eprintln!("smemcap: {}", ee);
                return Ok(1);
            }
        }
        Ok(0)
    }
}
