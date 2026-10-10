use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RtcwakeApplet;

impl Applet for RtcwakeApplet {
    fn name(&self) -> &'static str {
        "rtcwake"
    }
    fn description(&self) -> &'static str {
        "rtcwake"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::RtcwakeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

