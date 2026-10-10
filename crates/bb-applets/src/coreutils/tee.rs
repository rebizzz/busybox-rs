use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TeeApplet;

impl Applet for TeeApplet {
    fn name(&self) -> &'static str {
        "tee"
    }
    fn description(&self) -> &'static str {
        "tee"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::stream::TeeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

