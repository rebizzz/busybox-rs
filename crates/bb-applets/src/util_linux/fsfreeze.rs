use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FsfreezeApplet;

impl Applet for FsfreezeApplet {
    fn name(&self) -> &'static str {
        "fsfreeze"
    }
    fn description(&self) -> &'static str {
        "fsfreeze"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::FsfreezeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

