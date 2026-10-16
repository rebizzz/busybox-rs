use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PowertopApplet;

impl Applet for PowertopApplet {
    fn name(&self) -> &'static str {
        "powertop"
    }
    fn description(&self) -> &'static str {
        "powertop"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::PowertopApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

