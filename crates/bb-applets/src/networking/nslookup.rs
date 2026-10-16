use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NslookupApplet;

impl Applet for NslookupApplet {
    fn name(&self) -> &'static str {
        "nslookup"
    }
    fn description(&self) -> &'static str {
        "nslookup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::NslookupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

