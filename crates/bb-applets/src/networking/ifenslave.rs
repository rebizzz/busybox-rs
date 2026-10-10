use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IfenslaveApplet;

impl Applet for IfenslaveApplet {
    fn name(&self) -> &'static str {
        "ifenslave"
    }
    fn description(&self) -> &'static str {
        "ifenslave"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IfenslaveApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

