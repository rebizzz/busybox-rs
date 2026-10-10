use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TtyApplet;

impl Applet for TtyApplet {
    fn name(&self) -> &'static str {
        "tty"
    }
    fn description(&self) -> &'static str {
        "tty"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::TtyApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

