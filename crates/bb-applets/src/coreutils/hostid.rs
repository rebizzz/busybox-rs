use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HostidApplet;

impl Applet for HostidApplet {
    fn name(&self) -> &'static str {
        "hostid"
    }
    fn description(&self) -> &'static str {
        "hostid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::HostidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

