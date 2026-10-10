use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::PathBuf;

pub struct ScriptreplayApplet;
impl Applet for ScriptreplayApplet {
    fn name(&self) -> &'static str {
        "scriptreplay"
    }
    fn description(&self) -> &'static str {
        "Replay typescript using timing file (scriptreplay --timing f typescript)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut timing: Option<PathBuf> = None;
        let mut tspath: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"--timing" || b == b"-t") && i + 1 < args.len() {
                timing = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            } else if !b.starts_with(b"-") {
                tspath = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }
        let (timing, tspath) = match (timing, tspath) {
            (Some(t), Some(s)) => (t, s),
            _ => {
                eprintln!("scriptreplay: usage: scriptreplay --timing timingfile typescript");
                return Ok(1);
            }
        };
        let tdata = match std::fs::read_to_string(&timing) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("scriptreplay: {}: {e}", timing.display());
                return Ok(1);
            }
        };
        let tsdata = match std::fs::read(&tspath) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("scriptreplay: {}: {e}", tspath.display());
                return Ok(1);
            }
        };
        let out = std::io::stdout();
        let mut lock = out.lock();
        let mut off = 0usize;
        for line in tdata.lines() {
            let mut it = line.split_whitespace();
            let (Some(delay), Some(size)) = (it.next(), it.next()) else {
                continue;
            };
            let delay: f64 = delay.parse().unwrap_or(0.0);
            let size: usize = size.parse().unwrap_or(0);
            if delay > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(delay.min(5.0)));
            }
            let end = (off + size).min(tsdata.len());
            if lock.write_all(&tsdata[off..end]).is_err() {
                return Ok(1);
            }
            let _ = lock.flush();
            off = end;
        }
        if off < tsdata.len() && lock.write_all(&tsdata[off..]).is_err() {
            return Ok(1);
        }
        Ok(0)
    }
}
