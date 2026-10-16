use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BootchartdApplet;

impl Applet for BootchartdApplet {
    fn name(&self) -> &'static str {
        "bootchartd"
    }
    fn description(&self) -> &'static str {
        "bootchartd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::BootchartdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

