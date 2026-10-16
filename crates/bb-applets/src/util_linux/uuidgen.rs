use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UuidgenApplet;

impl Applet for UuidgenApplet {
    fn name(&self) -> &'static str {
        "uuidgen"
    }
    fn description(&self) -> &'static str {
        "uuidgen"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::UuidgenApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

