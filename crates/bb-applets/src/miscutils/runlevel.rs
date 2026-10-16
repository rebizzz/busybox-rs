use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RunlevelApplet;

impl Applet for RunlevelApplet {
    fn name(&self) -> &'static str {
        "runlevel"
    }
    fn description(&self) -> &'static str {
        "runlevel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::RunlevelApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

