use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DnsdomainnameApplet;

impl Applet for DnsdomainnameApplet {
    fn name(&self) -> &'static str {
        "dnsdomainname"
    }
    fn description(&self) -> &'static str {
        "dnsdomainname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::DnsdomainnameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct HostnameApplet;

impl Applet for HostnameApplet {
    fn name(&self) -> &'static str {
        "hostname"
    }
    fn description(&self) -> &'static str {
        "hostname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::HostnameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

