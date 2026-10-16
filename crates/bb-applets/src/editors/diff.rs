use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DiffApplet;

impl Applet for DiffApplet {
    fn name(&self) -> &'static str {
        "diff"
    }
    fn description(&self) -> &'static str {
        "diff"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::DiffApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

