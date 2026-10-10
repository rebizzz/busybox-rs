use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RdevApplet;

impl Applet for RdevApplet {
    fn name(&self) -> &'static str {
        "rdev"
    }
    fn description(&self) -> &'static str {
        "rdev"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::RdevApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

