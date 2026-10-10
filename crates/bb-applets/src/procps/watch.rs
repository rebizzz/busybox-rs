use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WatchApplet;

impl Applet for WatchApplet {
    fn name(&self) -> &'static str {
        "watch"
    }
    fn description(&self) -> &'static str {
        "watch"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::WatchApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

