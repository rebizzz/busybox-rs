use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TimeoutApplet;

impl Applet for TimeoutApplet {
    fn name(&self) -> &'static str {
        "timeout"
    }
    fn description(&self) -> &'static str {
        "timeout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::TimeoutApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

