use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WcApplet;

impl Applet for WcApplet {
    fn name(&self) -> &'static str {
        "wc"
    }
    fn description(&self) -> &'static str {
        "wc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text::WcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

