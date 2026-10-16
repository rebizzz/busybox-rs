use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MkdirApplet;

impl Applet for MkdirApplet {
    fn name(&self) -> &'static str {
        "mkdir"
    }
    fn description(&self) -> &'static str {
        "mkdir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::MkdirApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

