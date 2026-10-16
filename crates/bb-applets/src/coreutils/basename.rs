use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BasenameApplet;

impl Applet for BasenameApplet {
    fn name(&self) -> &'static str {
        "basename"
    }
    fn description(&self) -> &'static str {
        "basename"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::BasenameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

