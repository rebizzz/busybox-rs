use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ConspyApplet;

impl Applet for ConspyApplet {
    fn name(&self) -> &'static str {
        "conspy"
    }
    fn description(&self) -> &'static str {
        "conspy"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::ConspyApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

