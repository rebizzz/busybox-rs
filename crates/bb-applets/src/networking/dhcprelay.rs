use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DhcprelayApplet;

impl Applet for DhcprelayApplet {
    fn name(&self) -> &'static str {
        "dhcprelay"
    }
    fn description(&self) -> &'static str {
        "dhcprelay"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::DhcprelayApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

