use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MkfsMinixApplet;

impl Applet for MkfsMinixApplet {
    fn name(&self) -> &'static str {
        "mkfs_minix"
    }
    fn description(&self) -> &'static str {
        "mkfs_minix"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::MkfsMinixApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

