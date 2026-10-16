use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FstrimApplet;

impl Applet for FstrimApplet {
    fn name(&self) -> &'static str {
        "fstrim"
    }
    fn description(&self) -> &'static str {
        "fstrim"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::FstrimApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

