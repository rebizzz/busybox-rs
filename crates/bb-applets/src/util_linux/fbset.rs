use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FbsetApplet;

impl Applet for FbsetApplet {
    fn name(&self) -> &'static str {
        "fbset"
    }
    fn description(&self) -> &'static str {
        "fbset"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::FbsetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

