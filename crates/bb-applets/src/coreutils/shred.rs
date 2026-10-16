use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ShredApplet;

impl Applet for ShredApplet {
    fn name(&self) -> &'static str {
        "shred"
    }
    fn description(&self) -> &'static str {
        "shred"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::ShredApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

