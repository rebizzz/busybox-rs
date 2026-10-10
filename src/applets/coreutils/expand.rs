use crate::core::{Applet, Result};
use crate::core::fs::read_bytes_or_stdin;
use super::common::*;
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ExpandApplet;
impl Applet for ExpandApplet {
    fn name(&self) -> &'static str {
        "expand"
    }
    fn description(&self) -> &'static str {
        "Convert tabs to spaces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tab_size: usize = 8;
        let mut opt_initial = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-i" || bytes == b"--initial" {
                opt_initial = true;
            } else if bytes == b"-t" || bytes == b"--tabs" {
                if i + 1 < args.len() {
                    tab_size = args[i + 1].to_string_lossy().parse().unwrap_or(8);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-t") {
                tab_size = arg.to_string_lossy()[2..].parse().unwrap_or(8);
            } else if !bytes.starts_with(b"-") {
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
            let mut ptr_strbeg = 0;
            let mut ptr = 0;

            while ptr < content.len() {
                let c = content[ptr];
                if c == b'\n' {
                    handle.write_all(&content[ptr_strbeg..=ptr])?;
                    ptr += 1;
                    ptr_strbeg = ptr;
                    continue;
                }

                if opt_initial && c != b' ' && c != b'\t' {
                    while ptr < content.len() && content[ptr] != b'\n' {
                        ptr += 1;
                    }
                    continue;
                }

                if c == b'\t' {
                    let chunk = &content[ptr_strbeg..ptr];
                    handle.write_all(chunk)?;
                    let w = unicode_strwidth(chunk);
                    let spaces = tab_size - (w % tab_size);
                    for _ in 0..spaces {
                        handle.write_all(b" ")?;
                    }
                    ptr += 1;
                    ptr_strbeg = ptr;
                    continue;
                }
                ptr += 1;
            }

            if ptr_strbeg < content.len() {
                handle.write_all(&content[ptr_strbeg..])?;
            }
        }
        Ok(0)
    }
}
