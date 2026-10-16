use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Mke2fsApplet;

impl Applet for Mke2fsApplet {
    fn name(&self) -> &'static str {
        "mke2fs"
    }
    fn description(&self) -> &'static str {
        "mke2fs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::Mke2fsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct MkfsExt2Applet;

impl Applet for MkfsExt2Applet {
    fn name(&self) -> &'static str {
        "mkfs_ext2"
    }
    fn description(&self) -> &'static str {
        "mkfs_ext2"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::MkfsExt2Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

