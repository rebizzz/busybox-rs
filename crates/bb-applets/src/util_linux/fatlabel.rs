use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FatlabelApplet;

impl Applet for FatlabelApplet {
    fn name(&self) -> &'static str {
        "fatlabel"
    }
    fn description(&self) -> &'static str {
        "fatlabel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::FatlabelApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

