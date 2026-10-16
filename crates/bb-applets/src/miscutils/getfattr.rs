use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GetfattrApplet;

impl Applet for GetfattrApplet {
    fn name(&self) -> &'static str {
        "getfattr"
    }
    fn description(&self) -> &'static str {
        "getfattr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::GetfattrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

