use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NameifApplet;

impl Applet for NameifApplet {
    fn name(&self) -> &'static str {
        "nameif"
    }
    fn description(&self) -> &'static str {
        "nameif"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::NameifApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

