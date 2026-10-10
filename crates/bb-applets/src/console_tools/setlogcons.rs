use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetlogconsApplet;

impl Applet for SetlogconsApplet {
    fn name(&self) -> &'static str {
        "setlogcons"
    }
    fn description(&self) -> &'static str {
        "setlogcons"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SetlogconsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

