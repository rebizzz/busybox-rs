use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IfplugdApplet;

impl Applet for IfplugdApplet {
    fn name(&self) -> &'static str {
        "ifplugd"
    }
    fn description(&self) -> &'static str {
        "ifplugd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IfplugdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

