use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CpApplet;

impl Applet for CpApplet {
    fn name(&self) -> &'static str {
        "cp"
    }
    fn description(&self) -> &'static str {
        "cp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::CpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

