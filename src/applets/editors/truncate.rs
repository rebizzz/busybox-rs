use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct TruncateApplet;

impl Applet for TruncateApplet {
    fn name(&self) -> &'static str {
        "truncate"
    }
    fn description(&self) -> &'static str {
        "Shrink or extend the size of each FILE to the specified size"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut size: Option<u64> = None;
        let mut no_create = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" || b == b"--no-create" {
                no_create = true;
            } else if (b == b"-s" || b == b"--size") && i + 1 < args.len() {
                i += 1;
                size = parse_size(args[i].as_bytes());
            } else if b.starts_with(b"-s") && b.len() > 2 {
                size = parse_size(&b[2..]);
            } else if b.starts_with(b"-") {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        fn parse_size(b: &[u8]) -> Option<u64> {
            let s = std::str::from_utf8(b).ok()?;
            let s = s.trim();
            if s.is_empty() {
                return None;
            }
            let (num_part, mult) = if s.ends_with('K') || s.ends_with('k') {
                (&s[..s.len() - 1], 1024u64)
            } else if s.ends_with('M') || s.ends_with('m') {
                (&s[..s.len() - 1], 1024 * 1024u64)
            } else if s.ends_with('G') || s.ends_with('g') {
                (&s[..s.len() - 1], 1024 * 1024 * 1024u64)
            } else {
                (s, 1u64)
            };
            let val = num_part.parse::<u64>().ok()?;
            Some(val * mult)
        }

        let target_size = match size {
            Some(s) => s,
            None => {
                eprintln!("truncate: missing size option");
                return Ok(1);
            }
        };

        let mut ret = 0;
        for f in files {
            if no_create && !f.exists() {
                continue;
            }
            let file = OpenOptions::new().write(true).create(!no_create).open(f);
            match file {
                Ok(file) => {
                    if let Err(e) = file.set_len(target_size) {
                        eprintln!("truncate: {}: {}", f.display(), e);
                        ret = 1;
                    }
                }
                Err(e) => {
                    eprintln!("truncate: {}: {}", f.display(), e);
                    ret = 1;
                }
            }
        }

        Ok(ret)
    }
}

