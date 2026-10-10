use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChrtApplet;

impl Applet for ChrtApplet {
    fn name(&self) -> &'static str {
        "chrt"
    }
    fn description(&self) -> &'static str {
        "chrt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::ChrtApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

