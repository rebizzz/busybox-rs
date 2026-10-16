use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetconsoleApplet;

impl Applet for SetconsoleApplet {
    fn name(&self) -> &'static str {
        "setconsole"
    }
    fn description(&self) -> &'static str {
        "setconsole"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::SetconsoleApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

