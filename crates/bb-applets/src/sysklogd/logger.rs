use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LoggerApplet;

impl Applet for LoggerApplet {
    fn name(&self) -> &'static str {
        "logger"
    }
    fn description(&self) -> &'static str {
        "logger"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::LoggerApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

