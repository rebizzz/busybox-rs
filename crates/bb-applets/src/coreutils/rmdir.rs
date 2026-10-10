use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RmdirApplet;

impl Applet for RmdirApplet {
    fn name(&self) -> &'static str {
        "rmdir"
    }
    fn description(&self) -> &'static str {
        "rmdir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::RmdirApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

