use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PscanApplet;

impl Applet for PscanApplet {
    fn name(&self) -> &'static str {
        "pscan"
    }
    fn description(&self) -> &'static str {
        "pscan"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::PscanApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

