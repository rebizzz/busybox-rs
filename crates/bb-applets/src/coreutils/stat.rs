use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct StatApplet;

impl Applet for StatApplet {
    fn name(&self) -> &'static str {
        "stat"
    }
    fn description(&self) -> &'static str {
        "stat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::StatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

