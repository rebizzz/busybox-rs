use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IpcalcApplet;

impl Applet for IpcalcApplet {
    fn name(&self) -> &'static str {
        "ipcalc"
    }
    fn description(&self) -> &'static str {
        "ipcalc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::IpcalcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

