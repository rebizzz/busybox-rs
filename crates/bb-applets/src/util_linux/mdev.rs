use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MdevApplet;

impl Applet for MdevApplet {
    fn name(&self) -> &'static str {
        "mdev"
    }
    fn description(&self) -> &'static str {
        "mdev"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::MdevApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

