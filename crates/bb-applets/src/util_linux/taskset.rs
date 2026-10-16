use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TasksetApplet;

impl Applet for TasksetApplet {
    fn name(&self) -> &'static str {
        "taskset"
    }
    fn description(&self) -> &'static str {
        "taskset"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::TasksetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

