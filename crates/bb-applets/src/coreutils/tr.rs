use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TrApplet;

impl Applet for TrApplet {
    fn name(&self) -> &'static str {
        "tr"
    }
    fn description(&self) -> &'static str {
        "tr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::tr::TrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

