use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IprouteApplet;
impl Applet for IprouteApplet {
    fn name(&self) -> &'static str {
        "iproute"
    }
    fn description(&self) -> &'static str {
        "Manage routing table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_route(args)
    }
}
