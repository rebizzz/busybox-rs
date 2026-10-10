use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ZcipApplet;

impl Applet for ZcipApplet {
    fn name(&self) -> &'static str {
        "zcip"
    }
    fn description(&self) -> &'static str {
        "zcip"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::ZcipApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

