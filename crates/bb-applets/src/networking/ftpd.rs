use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FtpdApplet;

impl Applet for FtpdApplet {
    fn name(&self) -> &'static str {
        "ftpd"
    }
    fn description(&self) -> &'static str {
        "ftpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::FtpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

