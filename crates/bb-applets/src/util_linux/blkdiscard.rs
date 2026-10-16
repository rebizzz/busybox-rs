use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BlkdiscardApplet;

impl Applet for BlkdiscardApplet {
    fn name(&self) -> &'static str {
        "blkdiscard"
    }
    fn description(&self) -> &'static str {
        "blkdiscard"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::BlkdiscardApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

