use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct Traceroute6Applet;
impl Applet for Traceroute6Applet {
    fn name(&self) -> &'static str {
        "traceroute6"
    }
    fn description(&self) -> &'static str {
        "Trace the route to a host over IPv6"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_traceroute_generic(args, true)
    }
}
