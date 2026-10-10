use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub struct TimeApplet;

impl Applet for TimeApplet {
    fn name(&self) -> &'static str {
        "time"
    }
    fn description(&self) -> &'static str {
        "Time a simple command or give resource usage"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut idx = 0;
        let mut verbose = false;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if b == b"-p" || b == b"--portability" || b.starts_with(b"-") {
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            return Ok(0);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        let start = Instant::now();
        let mut ru_before: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_before);
        }

        let status = Command::new(cmd).args(cmd_args).status();

        let elapsed = start.elapsed();
        let mut ru_after: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_after);
        }

        let real_secs = elapsed.as_secs_f64();
        let user_secs = (ru_after.ru_utime.tv_sec - ru_before.ru_utime.tv_sec) as f64
            + (ru_after.ru_utime.tv_usec - ru_before.ru_utime.tv_usec) as f64 / 1_000_000.0;
        let sys_secs = (ru_after.ru_stime.tv_sec - ru_before.ru_stime.tv_sec) as f64
            + (ru_after.ru_stime.tv_usec - ru_before.ru_stime.tv_usec) as f64 / 1_000_000.0;

        if verbose {
            eprintln!("Command being timed: {:?}", cmd);
            eprintln!("User time (seconds): {:.2}", user_secs);
            eprintln!("System time (seconds): {:.2}", sys_secs);
            eprintln!("Elapsed (wall clock) time: {:.2}s", real_secs);
        } else {
            eprintln!("real\t{:.2}m{:.3}s", real_secs / 60.0, real_secs % 60.0);
            eprintln!("user\t{:.2}m{:.3}s", user_secs / 60.0, user_secs % 60.0);
            eprintln!("sys\t{:.2}m{:.3}s", sys_secs / 60.0, sys_secs % 60.0);
        }

        match status {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("time: {}: {}", Path::new(cmd).display(), e);
                Ok(127)
            }
        }
    }
}

