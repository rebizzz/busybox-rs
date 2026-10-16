use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HushApplet;

impl Applet for HushApplet {
    fn name(&self) -> &'static str {
        "hush"
    }
    fn description(&self) -> &'static str {
        "hush"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::HushApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

