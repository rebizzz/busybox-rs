use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IpcsApplet;

impl Applet for IpcsApplet {
    fn name(&self) -> &'static str {
        "ipcs"
    }
    fn description(&self) -> &'static str {
        "ipcs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::IpcsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

