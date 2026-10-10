use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CpioApplet;

impl Applet for CpioApplet {
    fn name(&self) -> &'static str {
        "cpio"
    }
    fn description(&self) -> &'static str {
        "cpio"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::CpioApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

