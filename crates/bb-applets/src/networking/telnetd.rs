use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TelnetdApplet;

impl Applet for TelnetdApplet {
    fn name(&self) -> &'static str {
        "telnetd"
    }
    fn description(&self) -> &'static str {
        "telnetd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::TelnetdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

