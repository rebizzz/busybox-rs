use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct VlockApplet;

impl Applet for VlockApplet {
    fn name(&self) -> &'static str {
        "vlock"
    }
    fn description(&self) -> &'static str {
        "vlock"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::VlockApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

