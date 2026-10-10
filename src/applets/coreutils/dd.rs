use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub struct DdApplet;

impl Applet for DdApplet {
    fn name(&self) -> &'static str {
        "dd"
    }
    fn description(&self) -> &'static str {
        "Convert and copy a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut if_path: Option<PathBuf> = None;
        let mut of_path: Option<PathBuf> = None;
        let mut bs = 512usize;
        let mut count: Option<usize> = None;
        let mut seek = 0u64;
        let mut skip = 0u64;
        let mut count_bytes = false;

        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"if=") {
                if_path = Some(PathBuf::from(OsStr::from_bytes(&b[3..])));
            } else if b.starts_with(b"of=") {
                of_path = Some(PathBuf::from(OsStr::from_bytes(&b[3..])));
            } else if b.starts_with(b"bs=") {
                bs = parse_dd_num(&b[3..]).unwrap_or(512);
            } else if b.starts_with(b"count=") {
                count = parse_dd_num(&b[6..]);
            } else if b.starts_with(b"seek=") {
                seek = parse_dd_num(&b[5..]).unwrap_or(0) as u64;
            } else if b.starts_with(b"skip=") {
                skip = parse_dd_num(&b[5..]).unwrap_or(0) as u64;
            } else if b.starts_with(b"iflag=") {
                for flag in b[6..].split(|&c| c == b',') {
                    if flag == b"count_bytes" {
                        count_bytes = true;
                    }
                }
            }
        }

        fn parse_dd_num(b: &[u8]) -> Option<usize> {
            let s = std::str::from_utf8(b).ok()?;
            let (num, mult) = if s.ends_with('k') || s.ends_with('K') {
                (&s[..s.len() - 1], 1024)
            } else if s.ends_with('M') || s.ends_with('m') {
                (&s[..s.len() - 1], 1024 * 1024)
            } else if s.ends_with('G') || s.ends_with('g') {
                (&s[..s.len() - 1], 1024 * 1024 * 1024)
            } else {
                (s, 1)
            };
            num.parse::<usize>().ok().map(|v| v * mult)
        }

        let mut input: Box<dyn Read> = match if_path {
            Some(p) => Box::new(open_or_stdin(&p)?),
            None => Box::new(io::stdin()),
        };

        if skip > 0 {
            let to_skip = skip * (bs as u64);
            let mut discarded = 0;
            let mut skip_buf = vec![0u8; 8192];
            while discarded < to_skip {
                let n = input.read(&mut skip_buf)?;
                if n == 0 {
                    break;
                }
                discarded += n as u64;
            }
        }

        let mut output: Box<dyn Write> = match of_path {
            Some(p) => {
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(seek == 0)
                    .open(&p)?;
                if seek > 0 {
                    f.seek(SeekFrom::Start(seek * bs as u64))?;
                }
                Box::new(f)
            }
            None => Box::new(io::stdout()),
        };

        let mut in_full = 0usize;
        let mut in_part = 0usize;
        let mut out_full = 0usize;
        let mut out_part = 0usize;
        let mut total_bytes = 0u64;

        let mut buf = vec![0u8; bs];
        let mut rem_bytes = if count_bytes { count } else { None };
        let mut rem_records = if !count_bytes { count } else { None };

        loop {
            if let Some(r) = rem_records {
                if r == 0 {
                    break;
                }
            }
            if let Some(b) = rem_bytes {
                if b == 0 {
                    break;
                }
            }
            let to_read = match rem_bytes {
                Some(b) => bs.min(b),
                None => bs,
            };
            let n = input.read(&mut buf[..to_read])?;
            if n == 0 {
                break;
            }
            if n == bs {
                in_full += 1;
            } else {
                in_part += 1;
            }
            output.write_all(&buf[..n])?;
            if n == bs {
                out_full += 1;
            } else {
                out_part += 1;
            }
            total_bytes += n as u64;
            if let Some(ref mut r) = rem_records {
                *r -= 1;
            }
            if let Some(ref mut b) = rem_bytes {
                *b -= n;
            }
        }

        output.flush()?;
        eprintln!("{}+{} records in", in_full, in_part);
        eprintln!("{}+{} records out", out_full, out_part);
        eprintln!("{} bytes copied", total_bytes);

        Ok(0)
    }
}

