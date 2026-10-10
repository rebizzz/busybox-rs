use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TsApplet;

impl Applet for TsApplet {
    fn name(&self) -> &'static str {
        "ts"
    }
    fn description(&self) -> &'static str {
        "ts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::TsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

