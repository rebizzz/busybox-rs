use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TrueApplet;

impl Applet for TrueApplet {
    fn name(&self) -> &'static str {
        "true"
    }
    fn description(&self) -> &'static str {
        "true"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::TrueApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

