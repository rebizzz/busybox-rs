use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IpcrmApplet;

impl Applet for IpcrmApplet {
    fn name(&self) -> &'static str {
        "ipcrm"
    }
    fn description(&self) -> &'static str {
        "ipcrm"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::IpcrmApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

