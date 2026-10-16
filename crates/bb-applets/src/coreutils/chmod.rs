use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChmodApplet;

impl Applet for ChmodApplet {
    fn name(&self) -> &'static str {
        "chmod"
    }
    fn description(&self) -> &'static str {
        "chmod"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::ChmodApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

