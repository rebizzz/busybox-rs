use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ReadaheadApplet;

impl Applet for ReadaheadApplet {
    fn name(&self) -> &'static str {
        "readahead"
    }
    fn description(&self) -> &'static str {
        "readahead"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::ReadaheadApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

