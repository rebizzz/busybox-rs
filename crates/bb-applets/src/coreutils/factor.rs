use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FactorApplet;

impl Applet for FactorApplet {
    fn name(&self) -> &'static str {
        "factor"
    }
    fn description(&self) -> &'static str {
        "factor"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::numbers::FactorApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

