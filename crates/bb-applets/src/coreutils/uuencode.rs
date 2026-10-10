use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UuencodeApplet;

impl Applet for UuencodeApplet {
    fn name(&self) -> &'static str {
        "uuencode"
    }
    fn description(&self) -> &'static str {
        "uuencode"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::encode::UuencodeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

