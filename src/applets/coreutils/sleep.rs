use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::thread;
use std::time::Duration;

pub struct SleepApplet;
impl Applet for SleepApplet {
    fn name(&self) -> &'static str {
        "sleep"
    }
    fn description(&self) -> &'static str {
        "Delay for a specified amount of time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("sleep: missing operand");
            return Ok(1);
        }
        let mut total_secs: f64 = 0.0;
        for arg in args {
            let s = arg.to_string_lossy();
            let (val_str, unit) = if let Some(stripped) = s.strip_suffix('s') {
                (stripped, 1.0)
            } else if let Some(stripped) = s.strip_suffix('m') {
                (stripped, 60.0)
            } else if let Some(stripped) = s.strip_suffix('h') {
                (stripped, 3600.0)
            } else if let Some(stripped) = s.strip_suffix('d') {
                (stripped, 86400.0)
            } else {
                (s.as_ref(), 1.0)
            };
            match val_str.parse::<f64>() {
                Ok(v) => total_secs += v * unit,
                Err(_) => {
                    eprintln!("sleep: invalid number '{}'", s);
                    return Ok(1);
                }
            }
        }
        thread::sleep(Duration::from_secs_f64(total_secs));
        Ok(0)
    }
}
