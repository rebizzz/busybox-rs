use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct JoinApplet;

impl Applet for JoinApplet {
    fn name(&self) -> &'static str {
        "join"
    }
    fn description(&self) -> &'static str {
        "Join lines of two files on a common field"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut field1 = 1usize;
        let mut field2 = 1usize;
        let mut delim: Option<u8> = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-1" && i + 1 < args.len() {
                i += 1;
                field1 = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
            } else if b == b"-2" && i + 1 < args.len() {
                i += 1;
                field2 = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
            } else if b == b"-j" && i + 1 < args.len() {
                i += 1;
                let f = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
                field1 = f;
                field2 = f;
            } else if b == b"-t" && i + 1 < args.len() {
                i += 1;
                if !args[i].is_empty() {
                    delim = Some(args[i].as_bytes()[0]);
                }
            } else if b.starts_with(b"-t") && b.len() > 2 {
                delim = Some(b[2]);
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(&args[i]);
            }
            i += 1;
        }

        if files.len() != 2 {
            eprintln!("join: requires exactly two files");
            return Ok(1);
        }

        type JoinedRecord = (Vec<u8>, Vec<Vec<u8>>);

        fn read_lines_split(
            path: &OsStr,
            f_idx: usize,
            delim: Option<u8>,
        ) -> Result<Vec<JoinedRecord>> {
            let r: Box<dyn BufRead> = if path == "-" {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(Path::new(path))?))
            };
            let mut out = Vec::new();
            for l in r.lines() {
                let l = l?;
                let b = l.into_bytes();
                let parts: Vec<Vec<u8>> = match delim {
                    Some(d) => b.split(|&c| c == d).map(|s| s.to_vec()).collect(),
                    None => b
                        .split(|&c| c == b' ' || c == b'\t')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_vec())
                        .collect(),
                };
                let key = if f_idx > 0 && f_idx <= parts.len() {
                    parts[f_idx - 1].clone()
                } else {
                    Vec::new()
                };
                out.push((key, parts));
            }
            Ok(out)
        }

        let l1 = match read_lines_split(files[0], field1, delim) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("join: {}", e);
                return Ok(1);
            }
        };
        let l2 = match read_lines_split(files[1], field2, delim) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("join: {}", e);
                return Ok(1);
            }
        };

        let sep = delim.unwrap_or(b' ');
        let out = io::stdout();
        let mut lock = out.lock();

        for (k1, p1) in &l1 {
            for (k2, p2) in &l2 {
                if k1 == k2 && !k1.is_empty() {
                    lock.write_all(k1)?;
                    for (idx, p) in p1.iter().enumerate() {
                        if idx + 1 != field1 {
                            lock.write_all(&[sep])?;
                            lock.write_all(p)?;
                        }
                    }
                    for (idx, p) in p2.iter().enumerate() {
                        if idx + 1 != field2 {
                            lock.write_all(&[sep])?;
                            lock.write_all(p)?;
                        }
                    }
                    lock.write_all(b"\n")?;
                }
            }
        }

        Ok(0)
    }
}

