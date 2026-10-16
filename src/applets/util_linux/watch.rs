use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct WatchApplet;
impl Applet for WatchApplet {
    fn name(&self) -> &'static str {
        "watch"
    }
    fn description(&self) -> &'static str {
        "Execute a program periodically, showing output fullscreen"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut interval = 2u64;
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" && i + 1 < args.len() {
                interval = args[i + 1].to_string_lossy().parse().unwrap_or(2);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        if cmd_idx >= args.len() {
            eprintln!("watch: [-n SEC] PROG [ARGS]");
            return Ok(1);
        }

        let prog = &args[cmd_idx];
        let prog_args = &args[cmd_idx + 1..];

        loop {
            print!("\x1b[2J\x1b[H");
            println!(
                "Every {}s: {}",
                interval,
                args[cmd_idx..]
                    .iter()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            println!();
            let _ = io::stdout().flush();

            let _ = Command::new(prog).args(prog_args).status();

            thread::sleep(Duration::from_secs(interval));
        }
    }
}
