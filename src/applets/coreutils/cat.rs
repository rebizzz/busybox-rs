use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct CatApplet;
impl Applet for CatApplet {
    fn name(&self) -> &'static str {
        "cat"
    }
    fn description(&self) -> &'static str {
        "Concatenate FILE(s) and print on standard output"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_ends = false;
        let mut number_all = false;
        let mut number_nonblank = false;
        let mut show_nonprinting = false;
        let mut files = Vec::new();
        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'e' => {
                            show_ends = true;
                            show_nonprinting = true;
                        }
                        b'E' => {
                            show_ends = true;
                        }
                        b'n' => {
                            number_all = true;
                        }
                        b'b' => {
                            number_nonblank = true;
                        }
                        b'v' => {
                            show_nonprinting = true;
                        }
                        _ => {}
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut line_num = 1;
        for file in files {
            let reader = open_or_stdin(file)?;
            let mut buf_reader = BufReader::new(reader);
            if !show_ends && !number_all && !number_nonblank && !show_nonprinting {
                io::copy(&mut buf_reader, &mut handle)?;
                continue;
            }
            let mut line_buf = Vec::new();
            while let Ok(n) = buf_reader.read_until(b'\n', &mut line_buf) {
                if n == 0 {
                    break;
                }
                let is_blank = line_buf.len() == 1 && line_buf[0] == b'\n';
                if number_nonblank {
                    if !is_blank {
                        write!(handle, "{:6}\t", line_num)?;
                        line_num += 1;
                    }
                } else if number_all {
                    write!(handle, "{:6}\t", line_num)?;
                    line_num += 1;
                }
                for &b in &line_buf {
                    if b == b'\n' {
                        if show_ends {
                            handle.write_all(b"$")?;
                        }
                        handle.write_all(b"\n")?;
                    } else if show_nonprinting {
                        if b < 32 && b != b'\t' {
                            handle.write_all(&[b'^', b + 64])?;
                        } else if b == 127 {
                            handle.write_all(b"^?")?;
                        } else if (128..160).contains(&b) {
                            handle.write_all(&[b'M', b'-', b'^', b - 128 + 64])?;
                        } else if (160..255).contains(&b) {
                            handle.write_all(&[b'M', b'-', b - 128])?;
                        } else if b == 255 {
                            handle.write_all(b"M-^?")?;
                        } else {
                            handle.write_all(&[b])?;
                        }
                    } else {
                        handle.write_all(&[b])?;
                    }
                }
                line_buf.clear();
            }
        }
        Ok(0)
    }
}
