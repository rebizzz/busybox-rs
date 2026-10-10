use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SslClientApplet;

impl Applet for SslClientApplet {
    fn name(&self) -> &'static str {
        "ssl_client"
    }
    fn description(&self) -> &'static str {
        "ssl_client"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::SslClientApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

