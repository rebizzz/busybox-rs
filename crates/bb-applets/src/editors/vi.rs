use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ViApplet;

impl Applet for ViApplet {
    fn name(&self) -> &'static str {
        "vi"
    }
    fn description(&self) -> &'static str {
        "vi"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::ViApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

