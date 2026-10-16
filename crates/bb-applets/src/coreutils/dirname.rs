use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DirnameApplet;

impl Applet for DirnameApplet {
    fn name(&self) -> &'static str {
        "dirname"
    }
    fn description(&self) -> &'static str {
        "dirname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::DirnameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

