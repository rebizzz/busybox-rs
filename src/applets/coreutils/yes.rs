use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct YesApplet;
impl Applet for YesApplet {
    fn name(&self) -> &'static str {
        "yes"
    }
    fn description(&self) -> &'static str {
        "Output a string repeatedly until killed"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let msg = if args.is_empty() {
            b"y".to_vec()
        } else {
            let mut out = Vec::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                out.extend_from_slice(a.as_bytes());
            }
            out
        };
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        loop {
            if handle.write_all(&msg).is_err() || handle.write_all(b"\n").is_err() {
                break;
            }
        }
        Ok(0)
    }
}
