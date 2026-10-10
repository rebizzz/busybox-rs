use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct Ping6Applet;
impl Applet for Ping6Applet {
    fn name(&self) -> &'static str {
        "ping6"
    }
    fn description(&self) -> &'static str {
        "Send ICMPv6 ECHO_REQUEST to network hosts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ping_generic(args, true)
    }
}
