use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetserialApplet;

impl Applet for SetserialApplet {
    fn name(&self) -> &'static str {
        "setserial"
    }
    fn description(&self) -> &'static str {
        "setserial"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SetserialApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

