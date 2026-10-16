use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DmesgApplet;

impl Applet for DmesgApplet {
    fn name(&self) -> &'static str {
        "dmesg"
    }
    fn description(&self) -> &'static str {
        "dmesg"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::DmesgApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

