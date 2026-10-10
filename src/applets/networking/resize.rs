use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, Read, Write};

pub struct ResizeApplet;

impl Applet for ResizeApplet {
    fn name(&self) -> &'static str {
        "resize"
    }
    fn description(&self) -> &'static str {
        "Determine and set terminal size"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let res = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };

        let (lines, cols) = if res == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
            (ws.ws_row, ws.ws_col)
        } else {
            print!("\x1b[7\x1b[r\x1b[999;999H\x1b[6n");
            let _ = io::stdout().flush();

            let mut resp = Vec::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut b = [0u8; 1];
            while let Ok(1) = handle.read(&mut b) {
                resp.push(b[0]);
                if b[0] == b'R' {
                    break;
                }
            }
            print!("\x1b[8");
            let _ = io::stdout().flush();

            let mut parsed = (24u16, 80u16);
            if let Ok(s) = std::str::from_utf8(&resp) {
                if let Some(pos) = s.find('[') {
                    let sub = &s[pos + 1..s.len().saturating_sub(1)];
                    let parts: Vec<&str> = sub.split(';').collect();
                    if parts.len() == 2 {
                        let r = parts[0].parse().unwrap_or(24);
                        let c = parts[1].parse().unwrap_or(80);
                        parsed = (r, c);
                    }
                }
            }
            parsed
        };

        println!("COLUMNS={};\nLINES={};\nexport COLUMNS LINES;", cols, lines);
        Ok(0)
    }
}
