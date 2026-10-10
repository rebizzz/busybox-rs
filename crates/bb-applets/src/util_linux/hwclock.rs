use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HwclockApplet;

impl Applet for HwclockApplet {
    fn name(&self) -> &'static str {
        "hwclock"
    }
    fn description(&self) -> &'static str {
        "hwclock"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::HwclockApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

