use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UnzipApplet;

impl Applet for UnzipApplet {
    fn name(&self) -> &'static str {
        "unzip"
    }
    fn description(&self) -> &'static str {
        "unzip"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UnzipApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

