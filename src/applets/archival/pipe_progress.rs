use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{Read, Write};

pub struct PipeProgressApplet;
impl Applet for PipeProgressApplet {
    fn name(&self) -> &'static str {
        "pipe_progress"
    }
    fn description(&self) -> &'static str {
        "Copy stdin to stdout showing byte progress on stderr"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let stdin = std::io::stdin();
        let mut inp = stdin.lock();
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        let mut total = 0u64;
        loop {
            let n = match inp.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("pipe_progress: {e}");
                    return Ok(1);
                }
            };
            if out.write_all(&buf[..n]).is_err() {
                return Ok(1);
            }
            total += n as u64;
            eprintln!("pipe_progress: {total} bytes");
        }
        Ok(0)
    }
}
