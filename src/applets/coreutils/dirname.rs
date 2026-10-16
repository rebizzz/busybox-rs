use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct DirnameApplet;
impl Applet for DirnameApplet {
    fn name(&self) -> &'static str {
        "dirname"
    }
    fn description(&self) -> &'static str {
        "Strip last component from file name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("dirname: missing operand");
            return Ok(1);
        }
        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.is_empty() {
                println!(".");
                continue;
            }
            let mut end = bytes.len();
            while end > 0 && bytes[end - 1] == b'/' {
                end -= 1;
            }
            if end == 0 {
                println!("/");
            } else {
                let trimmed = &bytes[..end];
                if let Some(pos) = trimmed.iter().rposition(|&b| b == b'/') {
                    let mut p_end = pos;
                    while p_end > 0 && trimmed[p_end - 1] == b'/' {
                        p_end -= 1;
                    }
                    if p_end == 0 {
                        println!("/");
                    } else {
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();
                        handle.write_all(&trimmed[..p_end])?;
                        handle.write_all(b"\n")?;
                    }
                } else {
                    println!(".");
                }
            }
        }
        Ok(0)
    }
}
