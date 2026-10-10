use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Udhcpc6Applet;

impl Applet for Udhcpc6Applet {
    fn name(&self) -> &'static str {
        "udhcpc6"
    }
    fn description(&self) -> &'static str {
        "udhcpc6"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::Udhcpc6Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

