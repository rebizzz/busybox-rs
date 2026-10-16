use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct EdApplet;

impl Applet for EdApplet {
    fn name(&self) -> &'static str {
        "ed"
    }
    fn description(&self) -> &'static str {
        "ed"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::EdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

