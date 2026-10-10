use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NprocApplet;

impl Applet for NprocApplet {
    fn name(&self) -> &'static str {
        "nproc"
    }
    fn description(&self) -> &'static str {
        "nproc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::NprocApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

