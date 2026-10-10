use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IostatApplet;

impl Applet for IostatApplet {
    fn name(&self) -> &'static str {
        "iostat"
    }
    fn description(&self) -> &'static str {
        "iostat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::IostatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

