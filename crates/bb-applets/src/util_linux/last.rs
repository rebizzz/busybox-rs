use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LastApplet;

impl Applet for LastApplet {
    fn name(&self) -> &'static str {
        "last"
    }
    fn description(&self) -> &'static str {
        "last"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::LastApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

