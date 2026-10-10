use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PwdApplet;

impl Applet for PwdApplet {
    fn name(&self) -> &'static str {
        "pwd"
    }
    fn description(&self) -> &'static str {
        "pwd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::PwdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

