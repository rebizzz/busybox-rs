use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UnlinkApplet;

impl Applet for UnlinkApplet {
    fn name(&self) -> &'static str {
        "unlink"
    }
    fn description(&self) -> &'static str {
        "unlink"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fs::UnlinkApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

