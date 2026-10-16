use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AsciiApplet;

impl Applet for AsciiApplet {
    fn name(&self) -> &'static str {
        "ascii"
    }
    fn description(&self) -> &'static str {
        "ascii"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::AsciiApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

