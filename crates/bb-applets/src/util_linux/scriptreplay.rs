use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ScriptreplayApplet;

impl Applet for ScriptreplayApplet {
    fn name(&self) -> &'static str {
        "scriptreplay"
    }
    fn description(&self) -> &'static str {
        "scriptreplay"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::ScriptreplayApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

