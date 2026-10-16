use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SysctlApplet;

impl Applet for SysctlApplet {
    fn name(&self) -> &'static str {
        "sysctl"
    }
    fn description(&self) -> &'static str {
        "sysctl"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SysctlApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

