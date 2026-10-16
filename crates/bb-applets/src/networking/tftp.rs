use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TftpApplet;

impl Applet for TftpApplet {
    fn name(&self) -> &'static str {
        "tftp"
    }
    fn description(&self) -> &'static str {
        "tftp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::TftpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct TftpdApplet;

impl Applet for TftpdApplet {
    fn name(&self) -> &'static str {
        "tftpd"
    }
    fn description(&self) -> &'static str {
        "tftpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::TftpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

