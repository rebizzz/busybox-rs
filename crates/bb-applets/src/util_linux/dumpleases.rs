use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DumpleasesApplet;

impl Applet for DumpleasesApplet {
    fn name(&self) -> &'static str {
        "dumpleases"
    }
    fn description(&self) -> &'static str {
        "dumpleases"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::DumpleasesApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

