use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FallocateApplet;

impl Applet for FallocateApplet {
    fn name(&self) -> &'static str {
        "fallocate"
    }
    fn description(&self) -> &'static str {
        "fallocate"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::FallocateApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

