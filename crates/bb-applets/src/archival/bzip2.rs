use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Bzip2Applet;

impl Applet for Bzip2Applet {
    fn name(&self) -> &'static str {
        "bzip2"
    }
    fn description(&self) -> &'static str {
        "bzip2"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::Bzip2Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

