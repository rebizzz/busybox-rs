use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};

pub struct NmeterApplet;

impl Applet for NmeterApplet {
    fn name(&self) -> &'static str {
        "nmeter"
    }
    fn description(&self) -> &'static str {
        "Format and display system status information"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let fmt = if !args.is_empty() {
            args[0].to_string_lossy().into_owned()
        } else {
            "%c %m %d".to_string()
        };

        let mem = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let mut free_kb = 0u64;
        for line in mem.lines() {
            if line.starts_with("MemFree:") {
                free_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
        }

        let out_str = fmt
            .replace("%c", "cpu:0%")
            .replace("%m", &format!("mem:{}M", free_kb / 1024))
            .replace("%d", "disk:0k");

        println!("{}", out_str);
        Ok(0)
    }
}

