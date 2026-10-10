use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TelnetApplet;

impl Applet for TelnetApplet {
    fn name(&self) -> &'static str {
        "telnet"
    }
    fn description(&self) -> &'static str {
        "telnet"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::TelnetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

