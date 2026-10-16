use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IfupApplet;

impl Applet for IfupApplet {
    fn name(&self) -> &'static str {
        "ifup"
    }
    fn description(&self) -> &'static str {
        "ifup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IfupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct IfdownApplet;

impl Applet for IfdownApplet {
    fn name(&self) -> &'static str {
        "ifdown"
    }
    fn description(&self) -> &'static str {
        "ifdown"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::config::IfdownApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

