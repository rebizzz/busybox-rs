use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DateApplet;

impl Applet for DateApplet {
    fn name(&self) -> &'static str {
        "date"
    }
    fn description(&self) -> &'static str {
        "date"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::DateApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

