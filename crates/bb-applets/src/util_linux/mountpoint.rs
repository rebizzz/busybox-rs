use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MountpointApplet;

impl Applet for MountpointApplet {
    fn name(&self) -> &'static str {
        "mountpoint"
    }
    fn description(&self) -> &'static str {
        "mountpoint"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::MountpointApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

