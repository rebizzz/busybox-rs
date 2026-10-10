use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChownApplet;

impl Applet for ChownApplet {
    fn name(&self) -> &'static str {
        "chown"
    }
    fn description(&self) -> &'static str {
        "chown"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::ChownApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

