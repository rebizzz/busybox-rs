use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IpApplet;

impl Applet for IpApplet {
    fn name(&self) -> &'static str {
        "ip"
    }
    fn description(&self) -> &'static str {
        "ip"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IpaddrApplet;

impl Applet for IpaddrApplet {
    fn name(&self) -> &'static str {
        "ipaddr"
    }
    fn description(&self) -> &'static str {
        "ipaddr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IpaddrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IplinkApplet;

impl Applet for IplinkApplet {
    fn name(&self) -> &'static str {
        "iplink"
    }
    fn description(&self) -> &'static str {
        "iplink"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IplinkApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IpneighApplet;

impl Applet for IpneighApplet {
    fn name(&self) -> &'static str {
        "ipneigh"
    }
    fn description(&self) -> &'static str {
        "ipneigh"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IpneighApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IprouteApplet;

impl Applet for IprouteApplet {
    fn name(&self) -> &'static str {
        "iproute"
    }
    fn description(&self) -> &'static str {
        "iproute"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IprouteApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IpruleApplet;

impl Applet for IpruleApplet {
    fn name(&self) -> &'static str {
        "iprule"
    }
    fn description(&self) -> &'static str {
        "iprule"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IpruleApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IptunnelApplet;

impl Applet for IptunnelApplet {
    fn name(&self) -> &'static str {
        "iptunnel"
    }
    fn description(&self) -> &'static str {
        "iptunnel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IptunnelApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

