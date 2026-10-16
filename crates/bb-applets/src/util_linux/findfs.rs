use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FindfsApplet;

impl Applet for FindfsApplet {
    fn name(&self) -> &'static str {
        "findfs"
    }
    fn description(&self) -> &'static str {
        "findfs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::FindfsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

