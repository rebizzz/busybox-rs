use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SwitchRootApplet;

impl Applet for SwitchRootApplet {
    fn name(&self) -> &'static str {
        "switch_root"
    }
    fn description(&self) -> &'static str {
        "switch_root"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::SwitchRootApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

