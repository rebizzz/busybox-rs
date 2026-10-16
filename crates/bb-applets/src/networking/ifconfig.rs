use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IfconfigApplet;

impl Applet for IfconfigApplet {
    fn name(&self) -> &'static str {
        "ifconfig"
    }
    fn description(&self) -> &'static str {
        "ifconfig"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IfconfigApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

