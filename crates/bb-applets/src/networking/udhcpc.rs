use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UdhcpcApplet;

impl Applet for UdhcpcApplet {
    fn name(&self) -> &'static str {
        "udhcpc"
    }
    fn description(&self) -> &'static str {
        "udhcpc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::UdhcpcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

