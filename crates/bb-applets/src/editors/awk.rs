use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AwkApplet;

impl Applet for AwkApplet {
    fn name(&self) -> &'static str {
        "awk"
    }
    fn description(&self) -> &'static str {
        "awk"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::AwkApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

