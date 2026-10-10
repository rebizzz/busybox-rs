use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FdiskApplet;

impl Applet for FdiskApplet {
    fn name(&self) -> &'static str {
        "fdisk"
    }
    fn description(&self) -> &'static str {
        "fdisk"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::FdiskApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

