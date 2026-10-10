use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NanddumpApplet;

impl Applet for NanddumpApplet {
    fn name(&self) -> &'static str {
        "nanddump"
    }
    fn description(&self) -> &'static str {
        "nanddump"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::NanddumpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct NandwriteApplet;

impl Applet for NandwriteApplet {
    fn name(&self) -> &'static str {
        "nandwrite"
    }
    fn description(&self) -> &'static str {
        "nandwrite"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::NandwriteApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

