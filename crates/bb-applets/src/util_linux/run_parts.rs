use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RunPartsApplet;

impl Applet for RunPartsApplet {
    fn name(&self) -> &'static str {
        "run_parts"
    }
    fn description(&self) -> &'static str {
        "run_parts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::RunPartsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

