use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SmemcapApplet;

impl Applet for SmemcapApplet {
    fn name(&self) -> &'static str {
        "smemcap"
    }
    fn description(&self) -> &'static str {
        "smemcap"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::SmemcapApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

