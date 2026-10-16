use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::Write;

pub struct RdevApplet;
impl Applet for RdevApplet {
    fn name(&self) -> &'static str {
        "rdev"
    }
    fn description(&self) -> &'static str {
        "Print the root device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("rdev: setting devices is not supported");
            return Ok(1);
        }
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::stat(c"/".as_ptr(), &mut st) } != 0 {
            eprintln!("rdev: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        let maj = libc::major(st.st_dev);
        let min = libc::minor(st.st_dev);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(48);
        line.extend_from_slice(b"root device ");
        push_u64(&mut line, maj as u64);
        line.push(b':');
        push_u64(&mut line, min as u64);
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
