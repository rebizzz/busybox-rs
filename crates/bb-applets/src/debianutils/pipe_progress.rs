use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PipeProgressApplet;

impl Applet for PipeProgressApplet {
    fn name(&self) -> &'static str {
        "pipe_progress"
    }
    fn description(&self) -> &'static str {
        "pipe_progress"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::PipeProgressApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

