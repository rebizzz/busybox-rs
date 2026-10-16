use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SedApplet;

impl Applet for SedApplet {
    fn name(&self) -> &'static str {
        "sed"
    }
    fn description(&self) -> &'static str {
        "sed"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::SedApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

