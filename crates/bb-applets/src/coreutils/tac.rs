use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TacApplet;

impl Applet for TacApplet {
    fn name(&self) -> &'static str {
        "tac"
    }
    fn description(&self) -> &'static str {
        "tac"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::TacApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

