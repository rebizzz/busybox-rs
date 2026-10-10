use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GzipApplet;

impl Applet for GzipApplet {
    fn name(&self) -> &'static str {
        "gzip"
    }
    fn description(&self) -> &'static str {
        "gzip"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::GzipApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

