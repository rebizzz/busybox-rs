use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BlkidApplet;

impl Applet for BlkidApplet {
    fn name(&self) -> &'static str {
        "blkid"
    }
    fn description(&self) -> &'static str {
        "blkid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::BlkidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

