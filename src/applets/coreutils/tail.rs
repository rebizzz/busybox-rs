use crate::core::fs::{read_bytes_or_stdin};
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct TailApplet;
impl Applet for TailApplet {
    fn name(&self) -> &'static str {
        "tail"
    }
    fn description(&self) -> &'static str {
        "Output the last part of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count: usize = 10;
        let mut from_beginning = false;
        let mut byte_mode = false;
        let mut quiet = false;
        let mut verbose = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-c") {
                byte_mode = true;
                let val_str = if bytes.len() > 2 {
                    &bytes[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if val_str.starts_with(b"+") {
                    from_beginning = true;
                    count = String::from_utf8_lossy(&val_str[1..]).parse().unwrap_or(1);
                } else {
                    from_beginning = false;
                    let s = if val_str.starts_with(b"-") {
                        &val_str[1..]
                    } else {
                        val_str
                    };
                    count = String::from_utf8_lossy(s).parse().unwrap_or(10);
                }
            } else if bytes.starts_with(b"-n") {
                byte_mode = false;
                let val_str = if bytes.len() > 2 {
                    &bytes[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if val_str.starts_with(b"+") {
                    from_beginning = true;
                    count = String::from_utf8_lossy(&val_str[1..]).parse().unwrap_or(1);
                } else {
                    from_beginning = false;
                    let s = if val_str.starts_with(b"-") {
                        &val_str[1..]
                    } else {
                        val_str
                    };
                    count = String::from_utf8_lossy(s).parse().unwrap_or(10);
                }
            } else if bytes.starts_with(b"-")
                && bytes.len() > 1
                && bytes[1..].iter().all(|b| b.is_ascii_digit())
            {
                byte_mode = false;
                from_beginning = false;
                count = String::from_utf8_lossy(&bytes[1..]).parse().unwrap_or(10);
            } else if bytes.starts_with(b"+")
                && bytes.len() > 1
                && bytes[1..].iter().all(|b| b.is_ascii_digit())
            {
                byte_mode = false;
                from_beginning = true;
                count = String::from_utf8_lossy(&bytes[1..]).parse().unwrap_or(1);
            } else if bytes == b"-q" {
                quiet = true;
            } else if bytes == b"-v" {
                verbose = true;
            } else if bytes == b"--" {
                i += 1;
                while i < args.len() {
                    files.push(Path::new(&args[i]));
                    i += 1;
                }
                break;
            } else if bytes.starts_with(b"-") && bytes != b"-" {
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let print_headers = (files.len() > 1 && !quiet) || verbose;
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for (idx, file) in files.iter().enumerate() {
            let content = match read_bytes_or_stdin(file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("tail: cannot open '{}': {}", file.display(), e);
                    continue;
                }
            };

            if print_headers {
                if idx > 0 {
                    handle.write_all(b"\n")?;
                }
                writeln!(handle, "==> {} <==", file.display())?;
            }

            if byte_mode {
                if from_beginning {
                    let start = count.saturating_sub(1);
                    if start < content.len() {
                        handle.write_all(&content[start..])?;
                    }
                } else {
                    let start = content.len().saturating_sub(count);
                    handle.write_all(&content[start..])?;
                }
            } else {
                if from_beginning {
                    let mut line_no = 1;
                    let mut offset = 0;
                    for (i, &b) in content.iter().enumerate() {
                        if line_no >= count {
                            offset = i;
                            break;
                        }
                        if b == b'\n' {
                            line_no += 1;
                            if line_no >= count {
                                offset = i + 1;
                                break;
                            }
                        }
                    }
                    if line_no >= count && offset < content.len() {
                        handle.write_all(&content[offset..])?;
                    }
                } else {
                    if count == 0 {
                        continue;
                    }
                    let mut newlines = 0;
                    let mut start = 0;
                    let mut found = false;
                    for (i, &b) in content.iter().enumerate().rev() {
                        if b == b'\n' {
                            if i + 1 == content.len() {
                                continue;
                            }
                            newlines += 1;
                            if newlines == count {
                                start = i + 1;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        start = 0;
                    }
                    handle.write_all(&content[start..])?;
                }
            }
        }
        Ok(0)
    }
}
