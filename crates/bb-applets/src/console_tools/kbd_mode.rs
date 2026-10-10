use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct KbdModeApplet;

impl Applet for KbdModeApplet {
    fn name(&self) -> &'static str {
        "kbd_mode"
    }
    fn description(&self) -> &'static str {
        "kbd_mode"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::KbdModeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

