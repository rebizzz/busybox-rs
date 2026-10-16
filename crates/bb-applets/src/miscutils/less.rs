use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LessApplet;

impl Applet for LessApplet {
    fn name(&self) -> &'static str {
        "less"
    }
    fn description(&self) -> &'static str {
        "less"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::LessApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

