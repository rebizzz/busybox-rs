use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UmountApplet;

impl Applet for UmountApplet {
    fn name(&self) -> &'static str {
        "umount"
    }
    fn description(&self) -> &'static str {
        "umount"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::UmountApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

