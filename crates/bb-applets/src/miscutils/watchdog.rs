use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WatchdogApplet;

impl Applet for WatchdogApplet {
    fn name(&self) -> &'static str {
        "watchdog"
    }
    fn description(&self) -> &'static str {
        "watchdog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::WatchdogApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

