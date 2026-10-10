use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CalApplet;

impl Applet for CalApplet {
    fn name(&self) -> &'static str {
        "cal"
    }
    fn description(&self) -> &'static str {
        "cal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::cal::CalApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

