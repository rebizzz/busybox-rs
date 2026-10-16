use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct EnvApplet;

impl Applet for EnvApplet {
    fn name(&self) -> &'static str {
        "env"
    }
    fn description(&self) -> &'static str {
        "env"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::EnvApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

