use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DevmemApplet;

impl Applet for DevmemApplet {
    fn name(&self) -> &'static str {
        "devmem"
    }
    fn description(&self) -> &'static str {
        "devmem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::DevmemApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

