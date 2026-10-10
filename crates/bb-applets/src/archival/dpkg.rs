use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DpkgApplet;

impl Applet for DpkgApplet {
    fn name(&self) -> &'static str {
        "dpkg"
    }
    fn description(&self) -> &'static str {
        "dpkg"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::DpkgApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

