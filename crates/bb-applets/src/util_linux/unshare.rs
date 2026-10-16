use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UnshareApplet;

impl Applet for UnshareApplet {
    fn name(&self) -> &'static str {
        "unshare"
    }
    fn description(&self) -> &'static str {
        "unshare"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::UnshareApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

