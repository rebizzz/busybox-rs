use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FalseApplet;

impl Applet for FalseApplet {
    fn name(&self) -> &'static str {
        "false"
    }
    fn description(&self) -> &'static str {
        "false"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::FalseApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

