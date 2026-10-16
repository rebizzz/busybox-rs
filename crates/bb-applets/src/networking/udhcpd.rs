use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UdhcpdApplet;

impl Applet for UdhcpdApplet {
    fn name(&self) -> &'static str {
        "udhcpd"
    }
    fn description(&self) -> &'static str {
        "udhcpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::UdhcpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

