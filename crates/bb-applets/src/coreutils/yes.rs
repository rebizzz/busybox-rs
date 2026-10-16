use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct YesApplet;

impl Applet for YesApplet {
    fn name(&self) -> &'static str {
        "yes"
    }
    fn description(&self) -> &'static str {
        "yes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::YesApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

