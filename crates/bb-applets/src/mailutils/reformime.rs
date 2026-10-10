use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ReformimeApplet;

impl Applet for ReformimeApplet {
    fn name(&self) -> &'static str {
        "reformime"
    }
    fn description(&self) -> &'static str {
        "reformime"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::ReformimeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

