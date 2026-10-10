use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WhoisApplet;

impl Applet for WhoisApplet {
    fn name(&self) -> &'static str {
        "whois"
    }
    fn description(&self) -> &'static str {
        "whois"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::WhoisApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

