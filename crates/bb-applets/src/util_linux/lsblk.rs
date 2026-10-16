use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsblkApplet;

impl Applet for LsblkApplet {
    fn name(&self) -> &'static str {
        "lsblk"
    }
    fn description(&self) -> &'static str {
        "lsblk"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::LsblkApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

