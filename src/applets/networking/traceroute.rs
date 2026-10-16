use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct TracerouteApplet;
impl Applet for TracerouteApplet {
    fn name(&self) -> &'static str {
        "traceroute"
    }
    fn description(&self) -> &'static str {
        "Trace the route to a host"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_traceroute_generic(args, false)
    }
}
