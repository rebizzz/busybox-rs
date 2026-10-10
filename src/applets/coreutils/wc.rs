use crate::core::fs::{read_bytes_or_stdin};
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct WcApplet;
impl Applet for WcApplet {
    fn name(&self) -> &'static str {
        "wc"
    }
    fn description(&self) -> &'static str {
        "Print newline, word, and byte counts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count_lines = false;
        let mut count_words = false;
        let mut count_chars = false;
        let mut max_line = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'l' => count_lines = true,
                        b'w' => count_words = true,
                        b'c' | b'm' => count_chars = true,
                        b'L' => max_line = true,
                        _ => {}
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
        }

        if !count_lines && !count_words && !count_chars && !max_line {
            count_lines = true;
            count_words = true;
            count_chars = true;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in &files {
            let content = read_bytes_or_stdin(file)?;
            let mut lines = 0usize;
            let mut words = 0usize;
            let chars = content.len();
            let mut longest = 0usize;
            let mut cur_line_len = 0usize;
            let mut in_word = false;

            for &b in &content {
                if b == b'\n' {
                    lines += 1;
                    if cur_line_len > longest {
                        longest = cur_line_len;
                    }
                    cur_line_len = 0;
                } else {
                    cur_line_len += 1;
                }

                if b.is_ascii_whitespace() {
                    in_word = false;
                } else if !in_word {
                    words += 1;
                    in_word = true;
                }
            }
            if cur_line_len > longest {
                longest = cur_line_len;
            }

            let mut parts = Vec::new();
            if count_lines {
                parts.push(format!("{}", lines));
            }
            if count_words {
                parts.push(format!("{}", words));
            }
            if count_chars {
                parts.push(format!("{}", chars));
            }
            if max_line {
                parts.push(format!("{}", longest));
            }

            if file.as_os_str() != "-" {
                parts.push(file.display().to_string());
            }
            writeln!(handle, "{}", parts.join(" "))?;
        }
        Ok(0)
    }
}

