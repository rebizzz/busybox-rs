use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetfattrApplet;

impl Applet for SetfattrApplet {
    fn name(&self) -> &'static str {
        "setfattr"
    }
    fn description(&self) -> &'static str {
        "setfattr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::SetfattrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

