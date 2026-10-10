use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AdjtimexApplet;

impl Applet for AdjtimexApplet {
    fn name(&self) -> &'static str {
        "adjtimex"
    }
    fn description(&self) -> &'static str {
        "adjtimex"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::AdjtimexApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

