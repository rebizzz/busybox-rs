use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::time::Instant;

pub struct TsApplet;

impl Applet for TsApplet {
    fn name(&self) -> &'static str {
        "ts"
    }
    fn description(&self) -> &'static str {
        "Timestamp standard input"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut rel = false;
        let mut format_str = b"%b %d %H:%M:%S".to_vec();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-s" {
                rel = true;
            } else if !b.starts_with(b"-") {
                format_str = b.to_vec();
            }
        }

        let start_time = Instant::now();
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut lock = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if rel {
                let elapsed = start_time.elapsed().as_secs();
                let hrs = elapsed / 3600;
                let mins = (elapsed % 3600) / 60;
                let secs = elapsed % 60;
                write!(lock, "{:02}:{:02}:{:02} ", hrs, mins, secs)?;
            } else {
                unsafe {
                    let mut tv: libc::timeval = std::mem::zeroed();
                    libc::gettimeofday(&mut tv, std::ptr::null_mut());
                    let mut tm: libc::tm = std::mem::zeroed();
                    libc::localtime_r(&tv.tv_sec, &mut tm);
                    let mut buf = [0u8; 128];
                    let cfmt = CString::new(format_str.clone()).unwrap_or_default();
                    let n = libc::strftime(
                        buf.as_mut_ptr() as *mut libc::c_char,
                        buf.len(),
                        cfmt.as_ptr(),
                        &tm,
                    );
                    if n > 0 {
                        lock.write_all(&buf[..n])?;
                        write!(lock, " ")?;
                    }
                }
            }
            lock.write_all(line.as_bytes())?;
            lock.write_all(b"\n")?;
        }

        Ok(0)
    }
}

