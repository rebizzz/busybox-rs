use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NmeterApplet;

impl Applet for NmeterApplet {
    fn name(&self) -> &'static str {
        "nmeter"
    }
    fn description(&self) -> &'static str {
        "nmeter"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::NmeterApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

