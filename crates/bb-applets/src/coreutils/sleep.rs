use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SleepApplet;

impl Applet for SleepApplet {
    fn name(&self) -> &'static str {
        "sleep"
    }
    fn description(&self) -> &'static str {
        "sleep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::SleepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

