use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ArpingApplet;

impl Applet for ArpingApplet {
    fn name(&self) -> &'static str {
        "arping"
    }
    fn description(&self) -> &'static str {
        "arping"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::ArpingApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

