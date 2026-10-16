use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NbdClientApplet;

impl Applet for NbdClientApplet {
    fn name(&self) -> &'static str {
        "nbd_client"
    }
    fn description(&self) -> &'static str {
        "nbd_client"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::NbdClientApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

