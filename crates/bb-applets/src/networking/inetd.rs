use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct InetdApplet;

impl Applet for InetdApplet {
    fn name(&self) -> &'static str {
        "inetd"
    }
    fn description(&self) -> &'static str {
        "inetd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::InetdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

