use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NsenterApplet;

impl Applet for NsenterApplet {
    fn name(&self) -> &'static str {
        "nsenter"
    }
    fn description(&self) -> &'static str {
        "nsenter"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::NsenterApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

