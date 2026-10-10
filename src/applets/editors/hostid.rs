use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct HostidApplet;

impl Applet for HostidApplet {
    fn name(&self) -> &'static str {
        "hostid"
    }
    fn description(&self) -> &'static str {
        "Print the numeric identifier for the current host"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let id = unsafe { libc::gethostid() };
        println!("{:08x}", id as u32);
        Ok(0)
    }
}
