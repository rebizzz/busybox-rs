use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UsleepApplet;

impl Applet for UsleepApplet {
    fn name(&self) -> &'static str {
        "usleep"
    }
    fn description(&self) -> &'static str {
        "usleep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::UsleepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

