use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BeepApplet;

impl Applet for BeepApplet {
    fn name(&self) -> &'static str {
        "beep"
    }
    fn description(&self) -> &'static str {
        "beep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::BeepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

