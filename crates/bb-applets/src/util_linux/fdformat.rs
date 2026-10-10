use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FdformatApplet;

impl Applet for FdformatApplet {
    fn name(&self) -> &'static str {
        "fdformat"
    }
    fn description(&self) -> &'static str {
        "fdformat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::FdformatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

