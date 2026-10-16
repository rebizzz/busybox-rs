use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct SslServerApplet;
impl Applet for SslServerApplet {
    fn name(&self) -> &'static str {
        "ssl_server"
    }
    fn description(&self) -> &'static str {
        "TLS server wrapper"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut prog: Vec<OsString> = Vec::new();
        let mut idx = 0;
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-f" {
                idx += 1;
            } else if !b.starts_with(b"-") {
                prog = args[idx..].to_vec();
                break;
            }
            idx += 1;
        }
        if prog.is_empty() {
            eprintln!("Usage: ssl_server -f PEMFILE PROG ARGS");
            return Ok(1);
        }
        let mut cmd = std::process::Command::new(&prog[0]);
        cmd.args(&prog[1..]);
        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("ssl_server: {}: {}", prog[0].to_string_lossy(), e);
                Ok(1)
            }
        }
    }
}
