use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FlockApplet;

impl Applet for FlockApplet {
    fn name(&self) -> &'static str {
        "flock"
    }
    fn description(&self) -> &'static str {
        "flock"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::FlockApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

