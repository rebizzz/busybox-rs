use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::time::Duration;

pub struct UsleepApplet;

impl Applet for UsleepApplet {
    fn name(&self) -> &'static str {
        "usleep"
    }
    fn description(&self) -> &'static str {
        "Sleep for the specified number of microseconds"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("usleep: missing operand");
            return Ok(1);
        }

        let micros: u64 = std::str::from_utf8(args[0].as_bytes())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        std::thread::sleep(Duration::from_micros(micros));
        Ok(0)
    }
}

