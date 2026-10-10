use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SumApplet;

impl Applet for SumApplet {
    fn name(&self) -> &'static str {
        "sum"
    }
    fn description(&self) -> &'static str {
        "sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::checksums::SumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

