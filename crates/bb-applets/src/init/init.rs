use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct InitApplet;

impl Applet for InitApplet {
    fn name(&self) -> &'static str {
        "init"
    }
    fn description(&self) -> &'static str {
        "init"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::InitApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct LinuxrcApplet;

impl Applet for LinuxrcApplet {
    fn name(&self) -> &'static str {
        "linuxrc"
    }
    fn description(&self) -> &'static str {
        "linuxrc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::LinuxrcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

