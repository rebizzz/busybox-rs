use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DuApplet;

impl Applet for DuApplet {
    fn name(&self) -> &'static str {
        "du"
    }
    fn description(&self) -> &'static str {
        "du"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::DuApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

