use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct HeadApplet;
impl Applet for HeadApplet {
    fn name(&self) -> &'static str {
        "head"
    }
    fn description(&self) -> &'static str {
        "Output the first part of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lines: Option<i64> = Some(10);
        let mut bytes_count: Option<i64> = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-n" {
                if i + 1 < args.len() {
                    lines = Some(args[i + 1].to_string_lossy().parse().unwrap_or(10));
                    bytes_count = None;
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                lines = Some(arg.to_string_lossy()[2..].parse().unwrap_or(10));
                bytes_count = None;
            } else if bytes == b"-c" {
                if i + 1 < args.len() {
                    bytes_count = Some(args[i + 1].to_string_lossy().parse().unwrap_or(0));
                    lines = None;
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-c") {
                bytes_count = Some(arg.to_string_lossy()[2..].parse().unwrap_or(0));
                lines = None;
            } else if bytes.starts_with(b"-")
                && bytes.len() > 1
                && bytes[1..].iter().all(|b| b.is_ascii_digit())
            {
                lines = Some(arg.to_string_lossy()[1..].parse().unwrap_or(10));
                bytes_count = None;
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let mut reader = open_or_stdin(file)?;

            if let Some(count) = bytes_count {
                if count >= 0 {
                    let mut limited = (&mut *reader).take(count as u64);
                    io::copy(&mut limited, &mut handle)?;
                } else {
                    let mut all_bytes = Vec::new();
                    let mut r = reader;
                    r.read_to_end(&mut all_bytes)?;
                    let keep = all_bytes.len().saturating_sub((-count) as usize);
                    handle.write_all(&all_bytes[..keep])?;
                }
            } else {
                let lines_num = lines.unwrap_or(10);
                let buf_reader = BufReader::new(reader);

                if lines_num >= 0 {
                    let mut count = 0;
                    for l in buf_reader.lines() {
                        if count >= lines_num {
                            break;
                        }
                        if let Ok(line) = l {
                            writeln!(handle, "{}", line)?;
                            count += 1;
                        }
                    }
                } else {
                    let all_lines: Vec<String> = buf_reader
                        .lines()
                        .map_while(std::result::Result::ok)
                        .collect();
                    let keep = all_lines.len().saturating_sub((-lines_num) as usize);
                    for l in &all_lines[..keep] {
                        writeln!(handle, "{}", l)?;
                    }
                }
            }
        }
        Ok(0)
    }
}

