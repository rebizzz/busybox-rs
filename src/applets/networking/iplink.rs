use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IplinkApplet;
impl Applet for IplinkApplet {
    fn name(&self) -> &'static str {
        "iplink"
    }
    fn description(&self) -> &'static str {
        "Manage network devices"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_link(args)
    }
}
