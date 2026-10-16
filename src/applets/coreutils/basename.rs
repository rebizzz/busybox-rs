use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct BasenameApplet;
impl Applet for BasenameApplet {
    fn name(&self) -> &'static str {
        "basename"
    }
    fn description(&self) -> &'static str {
        "Strip directory and suffix from filenames"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("basename: missing operand");
            return Ok(1);
        }
        let bytes = args[0].as_bytes();
        let mut end = bytes.len();
        while end > 0 && bytes[end - 1] == b'/' {
            end -= 1;
        }
        if end == 0 {
            println!("/");
            return Ok(0);
        }
        let trimmed = &bytes[..end];
        let name_bytes = if let Some(pos) = trimmed.iter().rposition(|&b| b == b'/') {
            &trimmed[pos + 1..]
        } else {
            trimmed
        };

        let mut out = name_bytes;
        if args.len() > 1 && !args[1].is_empty() {
            let suffix = args[1].as_bytes();
            if out.ends_with(suffix) && out.len() > suffix.len() {
                out = &out[..out.len() - suffix.len()];
            }
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(out)?;
        handle.write_all(b"\n")?;
        Ok(0)
    }
}
