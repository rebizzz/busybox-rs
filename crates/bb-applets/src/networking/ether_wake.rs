use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct EtherWakeApplet;

impl Applet for EtherWakeApplet {
    fn name(&self) -> &'static str {
        "ether_wake"
    }
    fn description(&self) -> &'static str {
        "ether_wake"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::EtherWakeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

