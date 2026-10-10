use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct TeeApplet;
impl Applet for TeeApplet {
    fn name(&self) -> &'static str {
        "tee"
    }
    fn description(&self) -> &'static str {
        "Copy standard input to each FILE and standard output"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut append = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes == b"-a" {
                append = true;
            } else if bytes == b"-i" {
            } else if !bytes.starts_with(b"-") {
                files.push(Path::new(arg));
            }
        }

        let mut handles = Vec::new();
        for f in &files {
            let file = if append {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(f)?
            } else {
                std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(f)?
            };
            handles.push(file);
        }

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();

        let mut buf = [0u8; 8192];
        loop {
            let n = match stdin_handle.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(e.into()),
            };
            stdout_handle.write_all(&buf[..n])?;
            for h in &mut handles {
                h.write_all(&buf[..n])?;
            }
        }
        Ok(0)
    }
}
