use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ResetApplet;

impl Applet for ResetApplet {
    fn name(&self) -> &'static str {
        "reset"
    }
    fn description(&self) -> &'static str {
        "reset"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::ResetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

