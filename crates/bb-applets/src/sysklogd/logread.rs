use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LogreadApplet;

impl Applet for LogreadApplet {
    fn name(&self) -> &'static str {
        "logread"
    }
    fn description(&self) -> &'static str {
        "logread"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::LogreadApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

