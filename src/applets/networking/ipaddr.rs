use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IpaddrApplet;
impl Applet for IpaddrApplet {
    fn name(&self) -> &'static str {
        "ipaddr"
    }
    fn description(&self) -> &'static str {
        "Manage IP addresses"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_addr(args)
    }
}
