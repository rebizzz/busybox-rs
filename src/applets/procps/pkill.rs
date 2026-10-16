use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct PkillApplet;
impl Applet for PkillApplet {
    fn name(&self) -> &'static str {
        "pkill"
    }
    fn description(&self) -> &'static str {
        "Signal processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (o, _) = match parse_pgrep(args, true) {
            Ok(v) => v,
            Err(rc) => return Ok(rc),
        };
        let m = collect_matches(&o);
        if m.is_empty() {
            return Ok(1);
        }
        let mut rc = 0;
        for (pid, _, _) in &m {
            if unsafe { libc::kill(*pid as i32, o.signal) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                eprintln!("pkill: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}
