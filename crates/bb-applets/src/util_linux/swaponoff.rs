use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SwaponApplet;

impl Applet for SwaponApplet {
    fn name(&self) -> &'static str {
        "swapon"
    }
    fn description(&self) -> &'static str {
        "swapon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SwaponApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SwapoffApplet;

impl Applet for SwapoffApplet {
    fn name(&self) -> &'static str {
        "swapoff"
    }
    fn description(&self) -> &'static str {
        "swapoff"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SwapoffApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

