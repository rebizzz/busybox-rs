use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MtApplet;

impl Applet for MtApplet {
    fn name(&self) -> &'static str {
        "mt"
    }
    fn description(&self) -> &'static str {
        "mt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::MtApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

