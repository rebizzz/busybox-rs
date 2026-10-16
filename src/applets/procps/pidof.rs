use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct PidofApplet;
impl Applet for PidofApplet {
    fn name(&self) -> &'static str {
        "pidof"
    }
    fn description(&self) -> &'static str {
        "Print PIDs of running processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut single = false;
        let mut omit: Vec<u32> = Vec::new();
        let mut names: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                single = true;
            } else if b == b"-x" {
            } else if b == b"-o" {
                i += 1;
                if i >= args.len() {
                    eprintln!("pidof: -o needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => omit.push(p),
                    None => {
                        eprintln!("pidof: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' {
                eprintln!("pidof: unknown option");
                return Ok(1);
            } else {
                names.push(b);
            }
            i += 1;
        }
        if names.is_empty() {
            eprintln!("usage: pidof [-s] [-o pid] name...");
            return Ok(1);
        }
        let mut found: Vec<u32> = Vec::new();
        let mut cmd = [0u8; 4096];
        for pid in proc_pids() {
            if omit.contains(&pid) {
                continue;
            }
            let st = match read_stat(pid) {
                Some(s) => s,
                None => continue,
            };
            let n = read_cmdline(pid, &mut cmd);
            let argv0base: Option<&[u8]> = if n > 0 {
                let end = cmd[..n].iter().position(|&c| c == 0).unwrap_or(n);
                let a0 = &cmd[..end];
                Some(match a0.iter().rposition(|&c| c == b'/') {
                    Some(p) => &a0[p + 1..],
                    None => a0,
                })
            } else {
                None
            };
            for want in &names {
                let wb = match want.iter().rposition(|&c| c == b'/') {
                    Some(p) => &want[p + 1..],
                    None => want,
                };
                if st.comm == *wb || argv0base == Some(wb) {
                    found.push(pid);
                    break;
                }
            }
        }
        if found.is_empty() {
            return Ok(1);
        }
        found.sort_unstable();
        if single {
            found.truncate(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(64);
        for (k, p) in found.iter().enumerate() {
            if k > 0 {
                line.push(b' ');
            }
            push_u64(&mut line, *p as u64);
        }
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
