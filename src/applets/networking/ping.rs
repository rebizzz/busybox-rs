use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct PingApplet;
impl Applet for PingApplet {
    fn name(&self) -> &'static str {
        "ping"
    }
    fn description(&self) -> &'static str {
        "Send ICMP ECHO_REQUEST to network hosts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ping_generic(args, false)
    }
}
