use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TimeApplet;

impl Applet for TimeApplet {
    fn name(&self) -> &'static str {
        "time"
    }
    fn description(&self) -> &'static str {
        "time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::TimeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

