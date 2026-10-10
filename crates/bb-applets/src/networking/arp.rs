use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ArpApplet;

impl Applet for ArpApplet {
    fn name(&self) -> &'static str {
        "arp"
    }
    fn description(&self) -> &'static str {
        "arp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::ArpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

