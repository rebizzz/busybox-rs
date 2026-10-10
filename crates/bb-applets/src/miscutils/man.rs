use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ManApplet;

impl Applet for ManApplet {
    fn name(&self) -> &'static str {
        "man"
    }
    fn description(&self) -> &'static str {
        "man"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::ManApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

