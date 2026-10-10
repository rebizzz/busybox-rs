use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::mem::MaybeUninit;

pub struct TtysizeApplet;
impl Applet for TtysizeApplet {
    fn name(&self) -> &'static str {
        "ttysize"
    }
    fn description(&self) -> &'static str {
        "Print dimensions of terminal or default to 80 24"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ws = MaybeUninit::<Winsize>::uninit();
        let (mut w, mut h) = (80u16, 24u16);

        for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                let ws = unsafe { ws.assume_init() };
                w = ws.ws_col;
                h = ws.ws_row;
                break;
            }
        }

        if args.is_empty() {
            println!("{} {}", w, h);
            return Ok(0);
        }

        let mut out = Vec::new();
        for arg in args {
            let s = arg.to_string_lossy();
            if s.contains('w') {
                out.push(format!("{}", w));
            }
            if s.contains('h') {
                out.push(format!("{}", h));
            }
        }

        println!("{}", out.join(" "));
        Ok(0)
    }
}
