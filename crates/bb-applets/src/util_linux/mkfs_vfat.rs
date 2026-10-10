use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MkdosfsApplet;

impl Applet for MkdosfsApplet {
    fn name(&self) -> &'static str {
        "mkdosfs"
    }
    fn description(&self) -> &'static str {
        "mkdosfs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::MkdosfsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct MkfsVfatApplet;

impl Applet for MkfsVfatApplet {
    fn name(&self) -> &'static str {
        "mkfs_vfat"
    }
    fn description(&self) -> &'static str {
        "mkfs_vfat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::MkfsVfatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

