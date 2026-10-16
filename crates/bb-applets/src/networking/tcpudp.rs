use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TcpsvdApplet;

impl Applet for TcpsvdApplet {
    fn name(&self) -> &'static str {
        "tcpsvd"
    }
    fn description(&self) -> &'static str {
        "tcpsvd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::TcpsvdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UdpsvdApplet;

impl Applet for UdpsvdApplet {
    fn name(&self) -> &'static str {
        "udpsvd"
    }
    fn description(&self) -> &'static str {
        "udpsvd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::sockets::UdpsvdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

