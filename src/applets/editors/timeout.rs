use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub struct TimeoutApplet;

impl Applet for TimeoutApplet {
    fn name(&self) -> &'static str {
        "timeout"
    }
    fn description(&self) -> &'static str {
        "Run a command with a time limit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("timeout: missing operand");
            return Ok(125);
        }

        let mut sig = libc::SIGTERM;
        let mut idx = 0;

        if args[0].as_bytes() == b"-s" && args.len() >= 3 {
            idx += 2;
            let sig_str = args[1].as_bytes();
            if sig_str == b"KILL" || sig_str == b"9" {
                sig = libc::SIGKILL;
            }
        }

        if idx >= args.len() {
            eprintln!("timeout: missing duration");
            return Ok(125);
        }

        let duration_bytes = args[idx].as_bytes();
        idx += 1;

        let duration_secs: u64 = {
            let s = std::str::from_utf8(duration_bytes).unwrap_or("0");
            let (num_str, mult) = if let Some(stripped) = s.strip_suffix('s') {
                (stripped, 1)
            } else if let Some(stripped) = s.strip_suffix('m') {
                (stripped, 60)
            } else if let Some(stripped) = s.strip_suffix('h') {
                (stripped, 3600)
            } else if let Some(stripped) = s.strip_suffix('d') {
                (stripped, 86400)
            } else {
                (s, 1)
            };
            num_str.parse::<u64>().unwrap_or(0) * mult
        };

        if idx >= args.len() {
            eprintln!("timeout: missing command");
            return Ok(125);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        let mut child = match Command::new(cmd).args(cmd_args).spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "timeout: failed to execute {}: {}",
                    Path::new(cmd).display(),
                    e
                );
                return Ok(127);
            }
        };

        let start = Instant::now();
        let timeout_dur = Duration::from_secs(duration_secs);

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Ok(status.code().unwrap_or(128));
                }
                Ok(None) => {
                    if start.elapsed() >= timeout_dur {
                        let pid = child.id() as i32;
                        unsafe {
                            libc::kill(pid, sig);
                        }
                        let _ = child.wait();
                        return Ok(124);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("timeout: error waiting for child: {}", e);
                    return Ok(125);
                }
            }
        }
    }
}

