use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LpdApplet;

impl Applet for LpdApplet {
    fn name(&self) -> &'static str {
        "lpd"
    }
    fn description(&self) -> &'static str {
        "lpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::LpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

