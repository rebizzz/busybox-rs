use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MktempApplet;

impl Applet for MktempApplet {
    fn name(&self) -> &'static str {
        "mktemp"
    }
    fn description(&self) -> &'static str {
        "mktemp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::MktempApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

