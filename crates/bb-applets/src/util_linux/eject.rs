use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct EjectApplet;

impl Applet for EjectApplet {
    fn name(&self) -> &'static str {
        "eject"
    }
    fn description(&self) -> &'static str {
        "eject"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::EjectApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

