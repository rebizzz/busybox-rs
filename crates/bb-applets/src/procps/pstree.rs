use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PstreeApplet;

impl Applet for PstreeApplet {
    fn name(&self) -> &'static str {
        "pstree"
    }
    fn description(&self) -> &'static str {
        "pstree"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::PstreeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

