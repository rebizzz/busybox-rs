use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SyslogdApplet;

impl Applet for SyslogdApplet {
    fn name(&self) -> &'static str {
        "syslogd"
    }
    fn description(&self) -> &'static str {
        "syslogd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::SyslogdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

