use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RmmodApplet;

impl Applet for RmmodApplet {
    fn name(&self) -> &'static str {
        "rmmod"
    }
    fn description(&self) -> &'static str {
        "rmmod"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::RmmodApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct ModprobeApplet;

impl Applet for ModprobeApplet {
    fn name(&self) -> &'static str {
        "modprobe"
    }
    fn description(&self) -> &'static str {
        "modprobe"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::ModprobeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

