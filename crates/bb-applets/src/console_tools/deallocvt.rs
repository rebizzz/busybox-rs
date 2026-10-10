use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DeallocvtApplet;

impl Applet for DeallocvtApplet {
    fn name(&self) -> &'static str {
        "deallocvt"
    }
    fn description(&self) -> &'static str {
        "deallocvt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::DeallocvtApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

