use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsusbApplet;

impl Applet for LsusbApplet {
    fn name(&self) -> &'static str {
        "lsusb"
    }
    fn description(&self) -> &'static str {
        "lsusb"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::LsusbApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

