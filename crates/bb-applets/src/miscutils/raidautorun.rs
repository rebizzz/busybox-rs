use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RaidautorunApplet;

impl Applet for RaidautorunApplet {
    fn name(&self) -> &'static str {
        "raidautorun"
    }
    fn description(&self) -> &'static str {
        "raidautorun"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::disk_fs::RaidautorunApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

