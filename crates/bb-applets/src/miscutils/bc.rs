use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DcApplet;

impl Applet for DcApplet {
    fn name(&self) -> &'static str {
        "dc"
    }
    fn description(&self) -> &'static str {
        "dc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::DcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct BcApplet;

impl Applet for BcApplet {
    fn name(&self) -> &'static str {
        "bc"
    }
    fn description(&self) -> &'static str {
        "bc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::BcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

