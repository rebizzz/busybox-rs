use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ShufApplet;

impl Applet for ShufApplet {
    fn name(&self) -> &'static str {
        "shuf"
    }
    fn description(&self) -> &'static str {
        "shuf"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::ShufApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

