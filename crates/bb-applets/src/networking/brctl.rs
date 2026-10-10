use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BrctlApplet;

impl Applet for BrctlApplet {
    fn name(&self) -> &'static str {
        "brctl"
    }
    fn description(&self) -> &'static str {
        "brctl"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::BrctlApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

