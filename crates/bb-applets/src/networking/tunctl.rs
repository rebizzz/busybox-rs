use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TunctlApplet;

impl Applet for TunctlApplet {
    fn name(&self) -> &'static str {
        "tunctl"
    }
    fn description(&self) -> &'static str {
        "tunctl"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::TunctlApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

