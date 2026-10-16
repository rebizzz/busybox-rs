use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TracerouteApplet;

impl Applet for TracerouteApplet {
    fn name(&self) -> &'static str {
        "traceroute"
    }
    fn description(&self) -> &'static str {
        "traceroute"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::TracerouteApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Traceroute6Applet;

impl Applet for Traceroute6Applet {
    fn name(&self) -> &'static str {
        "traceroute6"
    }
    fn description(&self) -> &'static str {
        "traceroute6"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::Traceroute6Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

