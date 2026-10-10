use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::Read;

pub struct WApplet;

impl Applet for WApplet {
    fn name(&self) -> &'static str {
        "w"
    }
    fn description(&self) -> &'static str {
        "Show who is logged on and what they are doing"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let uptime_data = fs::read_to_string("/proc/uptime").unwrap_or_default();
        let up_secs: f64 = uptime_data
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let hrs = (up_secs as u64) / 3600;
        let mins = ((up_secs as u64) % 3600) / 60;

        let load_data = fs::read_to_string("/proc/loadavg").unwrap_or_default();
        let loads: Vec<&str> = load_data.split_whitespace().take(3).collect();
        let load_str = loads.join(", ");

        println!(
            " up {:02}:{:02}, 1 user, load average: {}",
            hrs, mins, load_str
        );
        println!(
            "{:<8} {:<8} {:<10} {:<6} {:<6} {:<6} WHAT",
            "USER", "TTY", "FROM", "LOGIN@", "IDLE", "JCPU"
        );

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        println!(
            "{:<8} {:<8} {:<10} {:<6} {:<6} {:<6} busybox",
            user, "pts/0", "-", "00:00", "0.00s", "0.00s"
        );

        Ok(0)
    }
}

