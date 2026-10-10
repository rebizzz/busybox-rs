use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SplitApplet;

impl Applet for SplitApplet {
    fn name(&self) -> &'static str {
        "split"
    }
    fn description(&self) -> &'static str {
        "split"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::SplitApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

