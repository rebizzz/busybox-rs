use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct FuserApplet;

impl Applet for FuserApplet {
    fn name(&self) -> &'static str {
        "fuser"
    }
    fn description(&self) -> &'static str {
        "Identify processes using files or sockets"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut files = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") {
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.is_empty() {
            eprintln!("fuser: missing operand");
            return Ok(1);
        }

        let mut pids = Vec::new();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.chars().all(|c| c.is_ascii_digit()) {
                    let pid: i32 = match s.parse() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    let fd_dir = entry.path().join("fd");
                    if let Ok(fds) = fs::read_dir(fd_dir) {
                        for fd in fds.flatten() {
                            if let Ok(target) = fs::read_link(fd.path()) {
                                for target_f in &files {
                                    if target.ends_with(target_f) || target == **target_f {
                                        pids.push(pid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        pids.sort_unstable();
        pids.dedup();

        for pid in &pids {
            print!("{} ", pid);
        }
        if !pids.is_empty() {
            println!();
        }

        Ok(if pids.is_empty() { 1 } else { 0 })
    }
}

