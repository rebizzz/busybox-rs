use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WhichApplet;

impl Applet for WhichApplet {
    fn name(&self) -> &'static str {
        "which"
    }
    fn description(&self) -> &'static str {
        "which"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::WhichApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

