use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PopmaildirApplet;

impl Applet for PopmaildirApplet {
    fn name(&self) -> &'static str {
        "popmaildir"
    }
    fn description(&self) -> &'static str {
        "popmaildir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::PopmaildirApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

