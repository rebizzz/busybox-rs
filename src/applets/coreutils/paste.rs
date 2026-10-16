use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{self, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct PasteApplet;
impl Applet for PasteApplet {
    fn name(&self) -> &'static str {
        "paste"
    }
    fn description(&self) -> &'static str {
        "Merge lines of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delims = vec![b'\t'];
        let mut serial = false;
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1 && b[0] == b'-' && b != b"-" {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b's' => {
                            serial = true;
                            j += 1;
                        }
                        b'd' => {
                            let v = take_val(b, j, &mut i, args);
                            if v.is_empty() {
                                eprintln!("paste: no delimiters");
                                return Ok(1);
                            }
                            delims = parse_delims(&v);
                            break;
                        }
                        _ => j += 1,
                    }
                }
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut buf = Vec::with_capacity(4096);

        if serial {
            for f in &files {
                let mut input = if f.as_os_str() == "-" {
                    PasteInput::Stdin
                } else {
                    PasteInput::File(BufReader::new(std::fs::File::open(f)?))
                };
                let mut first = true;
                let mut di = 0;
                while next_line_paste(&mut input, &mut stdin_lock, &mut buf)? {
                    if !first {
                        let d = delims[di % delims.len()];
                        if d != 0 {
                            w.write_all(&[d])?;
                        }
                        di += 1;
                    }
                    first = false;
                    w.write_all(&buf)?;
                }
                if !first {
                    w.write_all(b"\n")?;
                }
            }
        } else {
            let mut inputs: Vec<Option<PasteInput>> = Vec::with_capacity(files.len());
            for f in &files {
                let input = if f.as_os_str() == "-" {
                    PasteInput::Stdin
                } else {
                    PasteInput::File(BufReader::new(std::fs::File::open(f)?))
                };
                inputs.push(Some(input));
            }

            let mut lines: Vec<Option<Vec<u8>>> = vec![None; inputs.len()];
            loop {
                let mut any = false;
                for k in 0..inputs.len() {
                    if let Some(ref mut inp) = inputs[k] {
                        if next_line_paste(inp, &mut stdin_lock, &mut buf)? {
                            any = true;
                            lines[k] = Some(buf.clone());
                        } else {
                            inputs[k] = None;
                            lines[k] = None;
                        }
                    } else {
                        lines[k] = None;
                    }
                }
                if !any {
                    break;
                }
                let mut del_idx = 0;
                for k in 0..inputs.len() {
                    if let Some(ref line_buf) = lines[k] {
                        w.write_all(line_buf)?;
                    }
                    let delim = if k == inputs.len() - 1 {
                        b'\n'
                    } else {
                        let d = delims[del_idx];
                        del_idx = (del_idx + 1) % delims.len();
                        d
                    };
                    if delim != 0 {
                        w.write_all(&[delim])?;
                    }
                }
            }
        }
        Ok(0)
    }
}
