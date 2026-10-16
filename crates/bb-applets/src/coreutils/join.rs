use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct JoinApplet;

impl Applet for JoinApplet {
    fn name(&self) -> &'static str {
        "join"
    }
    fn description(&self) -> &'static str {
        "join"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::JoinApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

