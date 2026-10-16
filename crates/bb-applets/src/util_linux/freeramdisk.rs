use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FdflushApplet;

impl Applet for FdflushApplet {
    fn name(&self) -> &'static str {
        "fdflush"
    }
    fn description(&self) -> &'static str {
        "fdflush"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::FdflushApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct FreeramdiskApplet;

impl Applet for FreeramdiskApplet {
    fn name(&self) -> &'static str {
        "freeramdisk"
    }
    fn description(&self) -> &'static str {
        "freeramdisk"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::FreeramdiskApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

