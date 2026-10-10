use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NiceApplet;

impl Applet for NiceApplet {
    fn name(&self) -> &'static str {
        "nice"
    }
    fn description(&self) -> &'static str {
        "nice"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::NiceApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

