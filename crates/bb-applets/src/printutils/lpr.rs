use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LpqApplet;

impl Applet for LpqApplet {
    fn name(&self) -> &'static str {
        "lpq"
    }
    fn description(&self) -> &'static str {
        "lpq"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::LpqApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct LprApplet;

impl Applet for LprApplet {
    fn name(&self) -> &'static str {
        "lpr"
    }
    fn description(&self) -> &'static str {
        "lpr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::LprApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

