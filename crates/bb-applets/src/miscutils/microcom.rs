use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MicrocomApplet;

impl Applet for MicrocomApplet {
    fn name(&self) -> &'static str {
        "microcom"
    }
    fn description(&self) -> &'static str {
        "microcom"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::MicrocomApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

