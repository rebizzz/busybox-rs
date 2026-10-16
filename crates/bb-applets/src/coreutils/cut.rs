use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CutApplet;

impl Applet for CutApplet {
    fn name(&self) -> &'static str {
        "cut"
    }
    fn description(&self) -> &'static str {
        "cut"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::cut::CutApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

