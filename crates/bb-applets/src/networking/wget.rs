use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WgetApplet;

impl Applet for WgetApplet {
    fn name(&self) -> &'static str {
        "wget"
    }
    fn description(&self) -> &'static str {
        "wget"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::WgetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

