use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct TacApplet;

impl Applet for TacApplet {
    fn name(&self) -> &'static str {
        "tac"
    }
    fn description(&self) -> &'static str {
        "Concatenate and print files in reverse line order"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut files: Vec<PathBuf> = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(PathBuf::from(arg));
            }
        }

        if files.is_empty() {
            files.push(PathBuf::from("-"));
        }

        let out = io::stdout();
        let mut lock = out.lock();

        for f in &files {
            let mut lines = Vec::new();
            if f == Path::new("-") {
                let stdin = io::stdin();
                for l in stdin.lock().lines() {
                    let mut b = l?.into_bytes();
                    b.push(b'\n');
                    lines.push(b);
                }
            } else {
                let data = match fs::read(f) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("tac: {}: {}", f.display(), e);
                        continue;
                    }
                };
                for l in data.split_inclusive(|&b| b == b'\n') {
                    lines.push(l.to_vec());
                }
            }

            for line in lines.iter().rev() {
                lock.write_all(line)?;
            }
        }

        Ok(0)
    }
}

