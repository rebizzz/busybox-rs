use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FbsplashApplet;

impl Applet for FbsplashApplet {
    fn name(&self) -> &'static str {
        "fbsplash"
    }
    fn description(&self) -> &'static str {
        "fbsplash"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::FbsplashApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

