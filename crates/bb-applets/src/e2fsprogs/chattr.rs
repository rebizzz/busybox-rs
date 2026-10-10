use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChattrApplet;

impl Applet for ChattrApplet {
    fn name(&self) -> &'static str {
        "chattr"
    }
    fn description(&self) -> &'static str {
        "chattr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::ChattrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

