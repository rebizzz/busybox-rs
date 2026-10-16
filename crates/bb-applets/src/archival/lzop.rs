use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LzopApplet;

impl Applet for LzopApplet {
    fn name(&self) -> &'static str {
        "lzop"
    }
    fn description(&self) -> &'static str {
        "lzop"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::LzopApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

