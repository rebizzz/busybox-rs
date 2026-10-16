use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NcApplet;

impl Applet for NcApplet {
    fn name(&self) -> &'static str {
        "nc"
    }
    fn description(&self) -> &'static str {
        "nc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::NcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

