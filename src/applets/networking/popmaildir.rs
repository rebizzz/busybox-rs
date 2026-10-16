use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub struct PopmaildirApplet;

impl Applet for PopmaildirApplet {
    fn name(&self) -> &'static str {
        "popmaildir"
    }
    fn description(&self) -> &'static str {
        "Fetch mail from POP3 server into maildir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dir = PathBuf::from(".");
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                dir = PathBuf::from(&args[i]);
            }
            i += 1;
        }

        let new_dir = dir.join("new");
        let _ = fs::create_dir_all(&new_dir);

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        let mut line = String::new();

        if reader.read_line(&mut line).is_err() {
            return Ok(1);
        }

        let _ = writer.write_all(b"STAT\r\n");
        let _ = writer.flush();
        line.clear();
        if reader.read_line(&mut line).is_err() || !line.starts_with("+OK") {
            return Ok(1);
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let msg_count: u32 = if parts.len() > 1 {
            parts[1].parse().unwrap_or(0)
        } else {
            0
        };

        for m in 1..=msg_count {
            let retr = format!("RETR {}\r\n", m);
            let _ = writer.write_all(retr.as_bytes());
            let _ = writer.flush();

            line.clear();
            if reader.read_line(&mut line).is_err() || !line.starts_with("+OK") {
                continue;
            }

            let msg_path = new_dir.join(format!("{}.msg", m));
            if let Ok(mut mf) = File::create(msg_path) {
                loop {
                    line.clear();
                    if reader.read_line(&mut line).is_err() {
                        break;
                    }
                    if line == ".\r\n" || line == ".\n" {
                        break;
                    }
                    let _ = mf.write_all(line.as_bytes());
                }
            }
        }

        let _ = writer.write_all(b"QUIT\r\n");
        let _ = writer.flush();
        Ok(0)
    }
}
