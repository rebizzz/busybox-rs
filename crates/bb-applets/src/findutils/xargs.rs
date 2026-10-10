use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct XargsApplet;

impl Applet for XargsApplet {
    fn name(&self) -> &'static str {
        "xargs"
    }
    fn description(&self) -> &'static str {
        "xargs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::shell::XargsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

