use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct KillallApplet;
impl Applet for KillallApplet {
    fn name(&self) -> &'static str {
        "killall"
    }
    fn description(&self) -> &'static str {
        "Send a signal to processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut quiet = false;
        let mut names: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-q" {
                quiet = true;
            } else if b == b"-l" {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                list_signals(&mut out)?;
                return Ok(0);
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                names.push(b);
            }
            i += 1;
        }
        if names.is_empty() {
            eprintln!("killall: usage: killall [-q] [-SIG] NAME...");
            return Ok(1);
        }

        let mut scratch = [0u8; 512];
        let mut matched: Vec<u32> = Vec::new();
        for_each_proc(&mut scratch, |p| {
            for want in &names {
                let wbase = match want.iter().rposition(|&c| c == b'/') {
                    Some(pos) => &want[pos + 1..],
                    None => want,
                };
                if p.comm == wbase {
                    matched.push(p.pid);
                    break;
                }
            }
        });
        let mut rc = 0;
        if matched.is_empty() {
            if !quiet {
                eprintln!("killall: no process killed");
            }
            return Ok(1);
        }
        for pid in matched {
            if unsafe { libc::kill(pid as i32, sig) } != 0 {
                eprintln!("killall: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}
