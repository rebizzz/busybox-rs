use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IptunnelApplet;
impl Applet for IptunnelApplet {
    fn name(&self) -> &'static str {
        "iptunnel"
    }
    fn description(&self) -> &'static str {
        "Manage IP tunnels"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_tunnel(args)
    }
}
