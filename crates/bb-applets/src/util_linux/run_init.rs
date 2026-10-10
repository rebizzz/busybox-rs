use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RunInitApplet;

impl Applet for RunInitApplet {
    fn name(&self) -> &'static str {
        "run_init"
    }
    fn description(&self) -> &'static str {
        "run_init"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::RunInitApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

