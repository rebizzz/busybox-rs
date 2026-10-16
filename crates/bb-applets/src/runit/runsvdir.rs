use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RunsvdirApplet;

impl Applet for RunsvdirApplet {
    fn name(&self) -> &'static str {
        "runsvdir"
    }
    fn description(&self) -> &'static str {
        "runsvdir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::RunsvdirApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

