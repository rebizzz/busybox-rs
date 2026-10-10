use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ScriptApplet;

impl Applet for ScriptApplet {
    fn name(&self) -> &'static str {
        "script"
    }
    fn description(&self) -> &'static str {
        "script"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::ScriptApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

