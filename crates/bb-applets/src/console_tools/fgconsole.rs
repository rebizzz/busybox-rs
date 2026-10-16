use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FgconsoleApplet;

impl Applet for FgconsoleApplet {
    fn name(&self) -> &'static str {
        "fgconsole"
    }
    fn description(&self) -> &'static str {
        "fgconsole"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::FgconsoleApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

