use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UbirenameApplet;

impl Applet for UbirenameApplet {
    fn name(&self) -> &'static str {
        "ubirename"
    }
    fn description(&self) -> &'static str {
        "ubirename"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbirenameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

