use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

pub struct NiceApplet;

impl Applet for NiceApplet {
    fn name(&self) -> &'static str {
        "nice"
    }
    fn description(&self) -> &'static str {
        "Run a program with modified scheduling priority"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut adjustment: i32 = 10;
        let mut idx = 0;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-n" && idx + 1 < args.len() {
                idx += 1;
                adjustment = std::str::from_utf8(args[idx].as_bytes())
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                adjustment = std::str::from_utf8(&b[2..])
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else if b.starts_with(b"-") && b.len() > 1 && (b[1].is_ascii_digit() || b[1] == b'-')
            {
                adjustment = std::str::from_utf8(&b[1..])
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            unsafe {
                let prio = libc::getpriority(libc::PRIO_PROCESS, 0);
                println!("{}", prio);
            }
            return Ok(0);
        }

        unsafe {
            let cur = libc::getpriority(libc::PRIO_PROCESS, 0);
            libc::setpriority(libc::PRIO_PROCESS, 0, cur + adjustment);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        match Command::new(cmd).args(cmd_args).status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("nice: {}: {}", Path::new(cmd).display(), e);
                Ok(127)
            }
        }
    }
}

