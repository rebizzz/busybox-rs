use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NohupApplet;

impl Applet for NohupApplet {
    fn name(&self) -> &'static str {
        "nohup"
    }
    fn description(&self) -> &'static str {
        "nohup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::NohupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

