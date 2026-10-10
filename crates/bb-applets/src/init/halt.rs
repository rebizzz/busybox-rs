use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HaltApplet;

impl Applet for HaltApplet {
    fn name(&self) -> &'static str {
        "halt"
    }
    fn description(&self) -> &'static str {
        "halt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::HaltApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct PoweroffApplet;

impl Applet for PoweroffApplet {
    fn name(&self) -> &'static str {
        "poweroff"
    }
    fn description(&self) -> &'static str {
        "poweroff"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::PoweroffApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct RebootApplet;

impl Applet for RebootApplet {
    fn name(&self) -> &'static str {
        "reboot"
    }
    fn description(&self) -> &'static str {
        "reboot"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::RebootApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

