use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SyncApplet;

impl Applet for SyncApplet {
    fn name(&self) -> &'static str {
        "sync"
    }
    fn description(&self) -> &'static str {
        "sync"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::SyncApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct FsyncApplet;

impl Applet for FsyncApplet {
    fn name(&self) -> &'static str {
        "fsync"
    }
    fn description(&self) -> &'static str {
        "fsync"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::FsyncApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

