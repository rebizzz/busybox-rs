use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NtpdApplet;

impl Applet for NtpdApplet {
    fn name(&self) -> &'static str {
        "ntpd"
    }
    fn description(&self) -> &'static str {
        "ntpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::NtpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

