use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MakedevsApplet;

impl Applet for MakedevsApplet {
    fn name(&self) -> &'static str {
        "makedevs"
    }
    fn description(&self) -> &'static str {
        "makedevs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::MakedevsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

