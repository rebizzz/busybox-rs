use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MoreApplet;

impl Applet for MoreApplet {
    fn name(&self) -> &'static str {
        "more"
    }
    fn description(&self) -> &'static str {
        "more"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::MoreApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

