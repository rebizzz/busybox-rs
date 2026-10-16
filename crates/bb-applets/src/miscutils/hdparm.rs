use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HdparmApplet;

impl Applet for HdparmApplet {
    fn name(&self) -> &'static str {
        "hdparm"
    }
    fn description(&self) -> &'static str {
        "hdparm"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::HdparmApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

