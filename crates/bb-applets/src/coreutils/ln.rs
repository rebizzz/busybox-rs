use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LnApplet;

impl Applet for LnApplet {
    fn name(&self) -> &'static str {
        "ln"
    }
    fn description(&self) -> &'static str {
        "ln"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::LnApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

