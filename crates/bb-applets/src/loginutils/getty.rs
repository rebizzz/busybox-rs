use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GettyApplet;

impl Applet for GettyApplet {
    fn name(&self) -> &'static str {
        "getty"
    }
    fn description(&self) -> &'static str {
        "getty"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::GettyApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

