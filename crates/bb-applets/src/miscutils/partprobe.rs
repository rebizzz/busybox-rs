use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PartprobeApplet;

impl Applet for PartprobeApplet {
    fn name(&self) -> &'static str {
        "partprobe"
    }
    fn description(&self) -> &'static str {
        "partprobe"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::PartprobeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

