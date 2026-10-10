use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LpqApplet;
impl Applet for LpqApplet {
    fn name(&self) -> &'static str {
        "lpq"
    }
    fn description(&self) -> &'static str {
        "Spool queue examination program"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut printer = "lp";
        for (i, arg) in args.iter().enumerate() {
            if arg.as_bytes() == b"-P" && i + 1 < args.len() {
                printer = args[i + 1].to_str().unwrap_or("lp");
            }
        }

        let spool_dir = Path::new("/var/spool/lpd").join(printer);
        println!("Printer: {}", printer);
        if !spool_dir.exists() {
            println!("no entries");
            return Ok(0);
        }

        let mut found = false;
        if let Ok(entries) = fs::read_dir(spool_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("df") || name.starts_with("cf") {
                    println!("Job: {}", name);
                    found = true;
                }
            }
        }

        if !found {
            println!("no entries");
        }

        Ok(0)
    }
}
