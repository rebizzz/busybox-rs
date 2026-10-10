use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct InstallApplet;

impl Applet for InstallApplet {
    fn name(&self) -> &'static str {
        "install"
    }
    fn description(&self) -> &'static str {
        "install"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::InstallApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

