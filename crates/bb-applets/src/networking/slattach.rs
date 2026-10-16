use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SlattachApplet;

impl Applet for SlattachApplet {
    fn name(&self) -> &'static str {
        "slattach"
    }
    fn description(&self) -> &'static str {
        "slattach"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::SlattachApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

