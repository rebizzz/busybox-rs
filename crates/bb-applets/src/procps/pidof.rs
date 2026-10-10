use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PidofApplet;

impl Applet for PidofApplet {
    fn name(&self) -> &'static str {
        "pidof"
    }
    fn description(&self) -> &'static str {
        "pidof"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::PidofApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

