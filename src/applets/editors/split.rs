use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct SplitApplet;

impl Applet for SplitApplet {
    fn name(&self) -> &'static str {
        "split"
    }
    fn description(&self) -> &'static str {
        "Split a file into pieces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lines_per_file: Option<usize> = None;
        let mut bytes_per_file: Option<usize> = None;
        let mut numeric_suffix = false;
        let mut file_arg: Option<&Path> = None;
        let mut prefix = b"x".to_vec();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                numeric_suffix = true;
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                lines_per_file = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-l") && b.len() > 2 {
                lines_per_file = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b == b"-b" && i + 1 < args.len() {
                i += 1;
                bytes_per_file = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-b") && b.len() > 2 {
                bytes_per_file = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-") && b.len() > 1 && b[1].is_ascii_digit() {
                lines_per_file = std::str::from_utf8(&b[1..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-") && b != b"-" {
            } else if file_arg.is_none() {
                file_arg = Some(Path::new(&args[i]));
            } else {
                prefix = b.to_vec();
            }
            i += 1;
        }

        let l_limit = lines_per_file.unwrap_or(if bytes_per_file.is_none() { 1000 } else { 0 });
        let input_path = file_arg.unwrap_or_else(|| Path::new("-"));

        let mut reader: Box<dyn Read> = if input_path == Path::new("-") {
            Box::new(io::stdin())
        } else {
            Box::new(open_or_stdin(input_path)?)
        };

        fn make_suffix(idx: usize, numeric: bool) -> Vec<u8> {
            if numeric {
                format!("{:02}", idx).into_bytes()
            } else {
                let first = (b'a' + (idx / 26) as u8) as char;
                let second = (b'a' + (idx % 26) as u8) as char;
                format!("{}{}", first, second).into_bytes()
            }
        }

        if let Some(b_limit) = bytes_per_file {
            let mut file_idx = 0;
            let mut buf = vec![0u8; b_limit];
            loop {
                let mut total_read = 0;
                while total_read < b_limit {
                    let n = reader.read(&mut buf[total_read..])?;
                    if n == 0 {
                        break;
                    }
                    total_read += n;
                }
                if total_read == 0 {
                    break;
                }
                let mut fname = prefix.clone();
                fname.extend_from_slice(&make_suffix(file_idx, numeric_suffix));
                fs::write(OsStr::from_bytes(&fname), &buf[..total_read])?;
                file_idx += 1;
            }
        } else {
            let mut buf_reader = BufReader::new(reader);
            let mut file_idx = 0;
            let mut line = Vec::new();
            let mut done = false;

            while !done {
                let mut cur_file_lines = 0;
                let mut fname = prefix.clone();
                fname.extend_from_slice(&make_suffix(file_idx, numeric_suffix));
                let mut cur_out: Option<File> = None;

                while cur_file_lines < l_limit {
                    line.clear();
                    let n = buf_reader.read_until(b'\n', &mut line)?;
                    if n == 0 {
                        done = true;
                        break;
                    }
                    if cur_out.is_none() {
                        cur_out = Some(File::create(OsStr::from_bytes(&fname))?);
                    }
                    if let Some(ref mut f) = cur_out {
                        f.write_all(&line)?;
                    }
                    cur_file_lines += 1;
                }
                if cur_out.is_some() {
                    file_idx += 1;
                }
            }
        }

        Ok(0)
    }
}

