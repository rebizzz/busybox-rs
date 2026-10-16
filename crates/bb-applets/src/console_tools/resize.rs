use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ResizeApplet;

impl Applet for ResizeApplet {
    fn name(&self) -> &'static str {
        "resize"
    }
    fn description(&self) -> &'static str {
        "resize"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::ResizeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

