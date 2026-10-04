use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};

pub struct XxdApplet;
impl Applet for XxdApplet {
    fn name(&self) -> &'static str { "xxd" }
    fn description(&self) -> &'static str { "Hexdump and reverse hexdump" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut plain = false;
        let mut revert = false;
        let mut cols: Option<usize> = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-p" || bytes == b"-ps" || bytes == b"-plain" {
                plain = true;
            } else if bytes == b"-r" {
                revert = true;
            } else if bytes == b"-c" {
                if i + 1 < args.len() {
                    cols = args[i + 1].to_string_lossy().parse().ok();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-c") {
                cols = arg.to_string_lossy()[2..].parse().ok();
            } else if bytes.starts_with(b"-p") {
                // In BusyBox: xxd accepts ANY string after -p (like -pc15) and ignores it!
                plain = true;
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        let file = if files.is_empty() { Path::new("-") } else { files[0] };
        let content = read_bytes_or_stdin(file)?;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        if revert {
            // Reverse hexdump
            let mut ptr = 0;
            let mut out = Vec::new();

            while ptr < content.len() {
                // Skip leading whitespace
                while ptr < content.len() && (content[ptr] == b' ' || content[ptr] == b'\t' || content[ptr] == b'\r' || content[ptr] == b'\n') {
                    ptr += 1;
                }
                if ptr >= content.len() { break; }

                if !plain {
                    // Skip address if present
                    if let Some(colon) = content[ptr..].iter().position(|&b| b == b':') {
                        ptr += colon + 1;
                    }
                }

                // Process line hex bytes
                while ptr < content.len() && content[ptr] != b'\n' {
                    if content[ptr] == b' ' || content[ptr] == b'\t' || content[ptr] == b'\r' {
                        if !plain && content[ptr] == b' ' && ptr + 1 < content.len() && content[ptr + 1] == b' ' {
                            // Two spaces: end of hex section, ASCII representation begins!
                            while ptr < content.len() && content[ptr] != b'\n' { ptr += 1; }
                            break;
                        }
                        ptr += 1;
                        continue;
                    }

                    // Check for truncation at two consecutive bad chars
                    if !content[ptr].is_ascii_hexdigit() {
                        if ptr + 1 < content.len() && !content[ptr + 1].is_ascii_hexdigit() && content[ptr + 1] != b'\n' {
                            while ptr < content.len() && content[ptr] != b'\n' { ptr += 1; }
                            break;
                        }
                        ptr += 1;
                        continue;
                    }

                    let hi = hex_val(content[ptr]);
                    ptr += 1;

                    // Allow whitespace between nibbles
                    while ptr < content.len() && (content[ptr] == b' ' || content[ptr] == b'\t' || content[ptr] == b'\r') {
                        ptr += 1;
                    }

                    if ptr < content.len() && content[ptr] != b'\n' {
                        if content[ptr].is_ascii_hexdigit() {
                            let lo = hex_val(content[ptr]);
                            ptr += 1;
                            out.push((hi << 4) | lo);
                        } else {
                            // 2nd nibble is bad: ignore this byte, skip until next hex or newline
                            while ptr < content.len() && content[ptr] != b'\n' && !content[ptr].is_ascii_hexdigit() {
                                ptr += 1;
                            }
                        }
                    }
                }
                if ptr < content.len() && content[ptr] == b'\n' {
                    ptr += 1;
                }
            }
            handle.write_all(&out)?;
            return Ok(0);
        }

        if plain {
            if content.is_empty() { return Ok(0); }
            let line_len = match cols {
                Some(0) => content.len(),
                Some(c) => c,
                None => 30,
            };

            let mut idx = 0;
            while idx < content.len() {
                let chunk_len = (content.len() - idx).min(line_len);
                let chunk = &content[idx..idx + chunk_len];
                for &b in chunk { write!(handle, "{:02x}", b)?; }
                writeln!(handle)?;
                idx += chunk_len;
            }
            return Ok(0);
        }

        // Standard xxd format
        let line_len = cols.unwrap_or(16);
        let mut idx = 0;
        while idx < content.len() {
            let chunk_len = (content.len() - idx).min(line_len);
            let chunk = &content[idx..idx + chunk_len];
            write!(handle, "{:08x}: ", idx)?;

            let mut j = 0;
            while j < chunk_len {
                write!(handle, "{:02x}", chunk[j])?;
                if j + 1 < chunk_len { write!(handle, "{:02x}", chunk[j + 1])?; }
                write!(handle, " ")?;
                j += 2;
            }
            let pad_bytes = line_len - chunk_len;
            let pad_groups = (pad_bytes + 1) / 2;
            for _ in 0..pad_groups { write!(handle, "     ")?; }

            write!(handle, " ")?;
            for &b in chunk {
                if b >= 32 && b < 127 { handle.write_all(&[b])?; } else { handle.write_all(b".")?; }
            }
            writeln!(handle)?;
            idx += chunk_len;
        }

        Ok(0)
    }
}

fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}
