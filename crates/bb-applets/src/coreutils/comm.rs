use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CommApplet;

impl Applet for CommApplet {
    fn name(&self) -> &'static str {
        "comm"
    }
    fn description(&self) -> &'static str {
        "comm"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::stream::CommApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

