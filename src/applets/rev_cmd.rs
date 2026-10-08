use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

pub struct RevApplet;
impl Applet for RevApplet {
    fn name(&self) -> &'static str {
        "rev"
    }
    fn description(&self) -> &'static str {
        "Reverse lines characterwise"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files = if args.is_empty() {
            vec![Path::new("-")]
        } else {
            args.iter().map(Path::new).collect()
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;
            let mut start = 0;
            let mut i = 0;
            while i < content.len() {
                if content[i] == b'\n' {
                    let line = &content[start..i];
                    // Standard C rev processes line with fgets/mbstowcs, which treats NUL as end of string
                    let (str_bytes, has_newline) =
                        if let Some(nul_pos) = line.iter().position(|&b| b == 0) {
                            (&line[..nul_pos], false)
                        } else {
                            (line, true)
                        };
                    let s = String::from_utf8_lossy(str_bytes);
                    let rev_s: String = s.chars().rev().collect();
                    handle.write_all(rev_s.as_bytes())?;
                    if has_newline {
                        handle.write_all(b"\n")?;
                    }
                    start = i + 1;
                }
                i += 1;
            }
            if start < content.len() {
                let line = &content[start..];
                let str_bytes = if let Some(nul_pos) = line.iter().position(|&b| b == 0) {
                    &line[..nul_pos]
                } else {
                    line
                };
                let s = String::from_utf8_lossy(str_bytes);
                let rev_s: String = s.chars().rev().collect();
                handle.write_all(rev_s.as_bytes())?;
            }
        }
        Ok(0)
    }
}
