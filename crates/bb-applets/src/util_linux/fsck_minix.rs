use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FsckMinixApplet;

impl Applet for FsckMinixApplet {
    fn name(&self) -> &'static str {
        "fsck_minix"
    }
    fn description(&self) -> &'static str {
        "fsck_minix"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::FsckMinixApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

