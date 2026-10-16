use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RmApplet;

impl Applet for RmApplet {
    fn name(&self) -> &'static str {
        "rm"
    }
    fn description(&self) -> &'static str {
        "rm"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::RmApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

