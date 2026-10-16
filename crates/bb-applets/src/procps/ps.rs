use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PsApplet;

impl Applet for PsApplet {
    fn name(&self) -> &'static str {
        "ps"
    }
    fn description(&self) -> &'static str {
        "ps"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::PsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

