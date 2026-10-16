use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RevApplet;

impl Applet for RevApplet {
    fn name(&self) -> &'static str {
        "rev"
    }
    fn description(&self) -> &'static str {
        "rev"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::rev::RevApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

