use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ResumeApplet;

impl Applet for ResumeApplet {
    fn name(&self) -> &'static str {
        "resume"
    }
    fn description(&self) -> &'static str {
        "resume"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::ResumeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

