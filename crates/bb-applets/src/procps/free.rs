use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FreeApplet;

impl Applet for FreeApplet {
    fn name(&self) -> &'static str {
        "free"
    }
    fn description(&self) -> &'static str {
        "free"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::FreeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

