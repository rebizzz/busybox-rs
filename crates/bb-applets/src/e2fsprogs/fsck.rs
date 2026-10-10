use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FsckApplet;

impl Applet for FsckApplet {
    fn name(&self) -> &'static str {
        "fsck"
    }
    fn description(&self) -> &'static str {
        "fsck"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::FsckApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

