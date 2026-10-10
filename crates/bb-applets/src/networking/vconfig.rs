use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct VconfigApplet;

impl Applet for VconfigApplet {
    fn name(&self) -> &'static str {
        "vconfig"
    }
    fn description(&self) -> &'static str {
        "vconfig"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::VconfigApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

