use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MountApplet;

impl Applet for MountApplet {
    fn name(&self) -> &'static str {
        "mount"
    }
    fn description(&self) -> &'static str {
        "mount"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::MountApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

