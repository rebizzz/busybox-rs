use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChatApplet;

impl Applet for ChatApplet {
    fn name(&self) -> &'static str {
        "chat"
    }
    fn description(&self) -> &'static str {
        "chat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::ChatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

