use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

pub struct LpdApplet;
impl Applet for LpdApplet {
    fn name(&self) -> &'static str {
        "lpd"
    }
    fn description(&self) -> &'static str {
        "Line printer spooling daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let spool_dir = if !args.is_empty() {
            PathBuf::from(&args[0])
        } else {
            PathBuf::from("/var/spool/lpd")
        };

        if !spool_dir.exists() {
            let _ = fs::create_dir_all(&spool_dir);
        }

        let mut stdin = io::stdin().lock();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_ok() {
            let _cmd = line.trim();

            let mut out = io::stdout().lock();
            let _ = out.write_all(b"\0");
            let _ = out.flush();
        }

        Ok(0)
    }
}
