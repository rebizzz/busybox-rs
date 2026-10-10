use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DdApplet;

impl Applet for DdApplet {
    fn name(&self) -> &'static str {
        "dd"
    }
    fn description(&self) -> &'static str {
        "dd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::DdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

