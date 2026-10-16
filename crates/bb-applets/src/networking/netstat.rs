use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NetstatApplet;

impl Applet for NetstatApplet {
    fn name(&self) -> &'static str {
        "netstat"
    }
    fn description(&self) -> &'static str {
        "netstat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::NetstatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

