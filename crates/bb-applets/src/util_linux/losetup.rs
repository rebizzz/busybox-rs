use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LosetupApplet;

impl Applet for LosetupApplet {
    fn name(&self) -> &'static str {
        "losetup"
    }
    fn description(&self) -> &'static str {
        "losetup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::LosetupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

