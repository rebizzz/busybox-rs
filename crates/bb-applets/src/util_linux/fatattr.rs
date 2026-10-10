use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FatattrApplet;

impl Applet for FatattrApplet {
    fn name(&self) -> &'static str {
        "fatattr"
    }
    fn description(&self) -> &'static str {
        "fatattr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::FatattrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

