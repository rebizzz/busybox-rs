use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PrintfApplet;

impl Applet for PrintfApplet {
    fn name(&self) -> &'static str {
        "printf"
    }
    fn description(&self) -> &'static str {
        "printf"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::printf::PrintfApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

