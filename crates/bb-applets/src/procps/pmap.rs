use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PmapApplet;

impl Applet for PmapApplet {
    fn name(&self) -> &'static str {
        "pmap"
    }
    fn description(&self) -> &'static str {
        "pmap"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::PmapApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

