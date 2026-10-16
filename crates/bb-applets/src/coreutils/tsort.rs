use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TsortApplet;

impl Applet for TsortApplet {
    fn name(&self) -> &'static str {
        "tsort"
    }
    fn description(&self) -> &'static str {
        "tsort"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::numbers::TsortApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

