use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CrontabApplet;

impl Applet for CrontabApplet {
    fn name(&self) -> &'static str {
        "crontab"
    }
    fn description(&self) -> &'static str {
        "crontab"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::CrontabApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

