use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TtysizeApplet;

impl Applet for TtysizeApplet {
    fn name(&self) -> &'static str {
        "ttysize"
    }
    fn description(&self) -> &'static str {
        "ttysize"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::TtysizeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

