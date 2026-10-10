use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TailApplet;

impl Applet for TailApplet {
    fn name(&self) -> &'static str {
        "tail"
    }
    fn description(&self) -> &'static str {
        "tail"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text::TailApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

