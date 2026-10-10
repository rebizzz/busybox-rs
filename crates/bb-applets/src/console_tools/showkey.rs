use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ShowkeyApplet;

impl Applet for ShowkeyApplet {
    fn name(&self) -> &'static str {
        "showkey"
    }
    fn description(&self) -> &'static str {
        "showkey"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::ShowkeyApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

