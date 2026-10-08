use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct FoldApplet;
impl Applet for FoldApplet {
    fn name(&self) -> &'static str {
        "fold"
    }
    fn description(&self) -> &'static str {
        "Wrap each input line to fit in specified width"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut width: usize = 80;
        let mut break_spaces = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-s" {
                break_spaces = true;
            } else if bytes == b"-w" {
                if i + 1 < args.len() {
                    width = args[i + 1].to_string_lossy().parse().unwrap_or(80);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-w") {
                width = arg.to_string_lossy()[2..].parse().unwrap_or(80);
            } else if bytes.starts_with(b"-sw") {
                break_spaces = true;
                width = arg.to_string_lossy()[3..].parse().unwrap_or(80);
            } else if bytes.starts_with(b"-")
                && bytes.len() > 1
                && bytes[1..].iter().all(|b| b.is_ascii_digit())
            {
                width = arg.to_string_lossy()[1..].parse().unwrap_or(80);
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
            let content = read_bytes_or_stdin(file)?;
            let mut line_out: Vec<u8> = Vec::new();
            let mut column: usize = 0;

            let mut idx = 0;
            while idx < content.len() {
                let c = content[idx];
                idx += 1;

                line_out.push(c);
                if c == b'\n' {
                    handle.write_all(&line_out)?;
                    line_out.clear();
                    column = 0;
                    continue;
                }

                // Adjust column according to BusyBox rule:
                // tabs advance to next tabstop (8)
                // backspace decrements column
                // utf-8 continuation bytes do not increment column
                if c == b'\t' {
                    column = column + 8 - (column % 8);
                } else if c == b'\x08' {
                    column = column.saturating_sub(1);
                } else if c == b'\r' {
                    column = 0;
                } else if (c & 0xc0) != 0x80 {
                    column += 1;
                }

                if column <= width || line_out.len() <= 1 {
                    continue;
                }

                // Overlong!
                if break_spaces {
                    // Search backwards in line_out[..line_out.len()-1] for space or tab
                    let mut found_blank = None;
                    for pos in (0..line_out.len() - 1).rev() {
                        if line_out[pos] == b' ' || line_out[pos] == b'\t' {
                            found_blank = Some(pos);
                            break;
                        }
                    }

                    if let Some(pos) = found_blank {
                        let logical_end = pos + 1;
                        handle.write_all(&line_out[..logical_end])?;
                        handle.write_all(b"\n")?;
                        let rem = line_out[logical_end..].to_vec();
                        line_out = rem;
                        // Recalculate column for line_out
                        column = 0;
                        for &b in &line_out {
                            if b == b'\t' {
                                column = column + 8 - (column % 8);
                            } else if b == b'\x08' {
                                column = column.saturating_sub(1);
                            } else if b == b'\r' {
                                column = 0;
                            } else if (b & 0xc0) != 0x80 {
                                column += 1;
                            }
                        }
                        continue;
                    }
                }

                // Split at current byte: output previous bytes + newline
                let cur = line_out.pop().unwrap();
                handle.write_all(&line_out)?;
                handle.write_all(b"\n")?;
                line_out.clear();
                line_out.push(cur);
                column = 0;
                if cur == b'\t' {
                    column = 8;
                } else if (cur & 0xc0) != 0x80 {
                    column = 1;
                }
            }

            if !line_out.is_empty() {
                handle.write_all(&line_out)?;
            }
        }
        Ok(0)
    }
}
