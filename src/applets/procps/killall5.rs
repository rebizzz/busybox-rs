use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct Killall5Applet;
impl Applet for Killall5Applet {
    fn name(&self) -> &'static str {
        "killall5"
    }
    fn description(&self) -> &'static str {
        "Signal all processes except caller"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut omit: Vec<u32> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-o" {
                i += 1;
                if i >= args.len() {
                    eprintln!("killall5: -o needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => omit.push(p),
                    None => {
                        eprintln!("killall5: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("killall5: -s needs a signal");
                    return Ok(1);
                }
                match sig_from_name(args[i].as_bytes()) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' {
                match std::str::from_utf8(&b[1..])
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("usage: killall5 [-SIGNAL] [-o pid] [-s sig]");
                return Ok(1);
            }
            i += 1;
        }
        let me = unsafe { libc::getpid() } as u32;
        let sid = unsafe { libc::getsid(0) } as u32;

        let mut rc = 2;

        if sig != libc::SIGSTOP && sig != libc::SIGCONT {
            unsafe { libc::kill(-1, libc::SIGSTOP) };
        }
        for pid in proc_pids() {
            if pid == me || pid == 1 || omit.contains(&pid) {
                continue;
            }

            let psid = unsafe { libc::getsid(pid as i32) } as u32;
            if psid == sid {
                continue;
            }
            unsafe { libc::kill(pid as i32, sig) };
            rc = 0;
        }
        if sig != libc::SIGSTOP && sig != libc::SIGCONT {
            unsafe { libc::kill(-1, libc::SIGCONT) };
        }
        Ok(rc)
    }
}
