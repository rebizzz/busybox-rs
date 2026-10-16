use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TarApplet;

impl Applet for TarApplet {
    fn name(&self) -> &'static str {
        "tar"
    }
    fn description(&self) -> &'static str {
        "tar"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::TarApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

