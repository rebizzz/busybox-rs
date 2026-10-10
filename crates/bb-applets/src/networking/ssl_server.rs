use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SslServerApplet;

impl Applet for SslServerApplet {
    fn name(&self) -> &'static str {
        "ssl_server"
    }
    fn description(&self) -> &'static str {
        "ssl_server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::SslServerApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

