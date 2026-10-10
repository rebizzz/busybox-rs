use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LinkApplet;

impl Applet for LinkApplet {
    fn name(&self) -> &'static str {
        "link"
    }
    fn description(&self) -> &'static str {
        "link"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::LinkApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

