use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct BlockdevApplet;

impl Applet for BlockdevApplet {
    fn name(&self) -> &'static str {
        "blockdev"
    }
    fn description(&self) -> &'static str {
        "blockdev"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::BlockdevApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

