use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GetoptApplet;

impl Applet for GetoptApplet {
    fn name(&self) -> &'static str {
        "getopt"
    }
    fn description(&self) -> &'static str {
        "getopt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::GetoptApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

