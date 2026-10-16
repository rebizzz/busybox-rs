use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TopApplet;

impl Applet for TopApplet {
    fn name(&self) -> &'static str {
        "top"
    }
    fn description(&self) -> &'static str {
        "top"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::TopApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

