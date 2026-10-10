use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::Write;

pub struct ReadprofileApplet;
impl Applet for ReadprofileApplet {
    fn name(&self) -> &'static str {
        "readprofile"
    }
    fn description(&self) -> &'static str {
        "Read kernel profiling data"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("usage: readprofile");
            return Ok(1);
        }

        let data = match std::fs::read("/proc/profile") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("readprofile: /proc/profile: {}", e);
                return Ok(1);
            }
        };
        if data.len() < 12 {
            eprintln!("readprofile: short profile");
            return Ok(1);
        }

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut addr: u64 = 0;
        let mut i = 4;
        while i + 4 <= data.len() {
            let hits = u32::from_ne_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as u64;
            if hits > 0 {
                let mut line = Vec::with_capacity(48);

                let mut v = addr;
                let mut rev = [0u8; 16];
                let mut rn = 0;
                if v == 0 {
                    rev[0] = b'0';
                    rn = 1;
                } else {
                    while v > 0 {
                        let d = (v & 0xf) as u8;
                        rev[rn] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                        v >>= 4;
                        rn += 1;
                    }
                }
                while rn > 0 {
                    rn -= 1;
                    line.push(rev[rn]);
                }
                line.push(b' ');
                push_u64(&mut line, hits);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            addr += 4;
            i += 4;
        }
        out.flush()?;
        Ok(0)
    }
}
