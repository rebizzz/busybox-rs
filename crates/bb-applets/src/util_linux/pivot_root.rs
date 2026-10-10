use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PivotRootApplet;

impl Applet for PivotRootApplet {
    fn name(&self) -> &'static str {
        "pivot_root"
    }
    fn description(&self) -> &'static str {
        "pivot_root"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::PivotRootApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

