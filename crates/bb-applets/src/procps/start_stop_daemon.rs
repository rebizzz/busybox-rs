use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct StartStopDaemonApplet;

impl Applet for StartStopDaemonApplet {
    fn name(&self) -> &'static str {
        "start_stop_daemon"
    }
    fn description(&self) -> &'static str {
        "start_stop_daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::StartStopDaemonApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

