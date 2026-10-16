use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PingApplet;

impl Applet for PingApplet {
    fn name(&self) -> &'static str {
        "ping"
    }
    fn description(&self) -> &'static str {
        "ping"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::PingApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Ping6Applet;

impl Applet for Ping6Applet {
    fn name(&self) -> &'static str {
        "ping6"
    }
    fn description(&self) -> &'static str {
        "ping6"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::Ping6Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

