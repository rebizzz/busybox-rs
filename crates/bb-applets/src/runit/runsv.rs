use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RunsvApplet;

impl Applet for RunsvApplet {
    fn name(&self) -> &'static str {
        "runsv"
    }
    fn description(&self) -> &'static str {
        "runsv"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::RunsvApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

