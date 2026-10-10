use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SwaplabelApplet;

impl Applet for SwaplabelApplet {
    fn name(&self) -> &'static str {
        "swaplabel"
    }
    fn description(&self) -> &'static str {
        "swaplabel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::SwaplabelApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

