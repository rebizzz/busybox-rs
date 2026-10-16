use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ReniceApplet;

impl Applet for ReniceApplet {
    fn name(&self) -> &'static str {
        "renice"
    }
    fn description(&self) -> &'static str {
        "renice"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::ReniceApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

