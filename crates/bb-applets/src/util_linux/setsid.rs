use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetsidApplet;

impl Applet for SetsidApplet {
    fn name(&self) -> &'static str {
        "setsid"
    }
    fn description(&self) -> &'static str {
        "setsid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::SetsidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

