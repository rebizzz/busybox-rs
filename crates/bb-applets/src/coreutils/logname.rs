use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LognameApplet;

impl Applet for LognameApplet {
    fn name(&self) -> &'static str {
        "logname"
    }
    fn description(&self) -> &'static str {
        "logname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::LognameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

