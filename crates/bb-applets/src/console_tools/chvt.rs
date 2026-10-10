use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChvtApplet;

impl Applet for ChvtApplet {
    fn name(&self) -> &'static str {
        "chvt"
    }
    fn description(&self) -> &'static str {
        "chvt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::ChvtApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

