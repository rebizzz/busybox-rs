use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RdateApplet;

impl Applet for RdateApplet {
    fn name(&self) -> &'static str {
        "rdate"
    }
    fn description(&self) -> &'static str {
        "rdate"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::RdateApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

