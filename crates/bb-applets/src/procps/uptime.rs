use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UptimeApplet;

impl Applet for UptimeApplet {
    fn name(&self) -> &'static str {
        "uptime"
    }
    fn description(&self) -> &'static str {
        "uptime"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::UptimeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

