use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TruncateApplet;

impl Applet for TruncateApplet {
    fn name(&self) -> &'static str {
        "truncate"
    }
    fn description(&self) -> &'static str {
        "truncate"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::TruncateApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

