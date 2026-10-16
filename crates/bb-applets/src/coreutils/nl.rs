use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NlApplet;

impl Applet for NlApplet {
    fn name(&self) -> &'static str {
        "nl"
    }
    fn description(&self) -> &'static str {
        "nl"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text2::NlApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

