use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Linux32Applet;

impl Applet for Linux32Applet {
    fn name(&self) -> &'static str {
        "linux32"
    }
    fn description(&self) -> &'static str {
        "linux32"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::Linux32Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Linux64Applet;

impl Applet for Linux64Applet {
    fn name(&self) -> &'static str {
        "linux64"
    }
    fn description(&self) -> &'static str {
        "linux64"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::Linux64Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SetarchApplet;

impl Applet for SetarchApplet {
    fn name(&self) -> &'static str {
        "setarch"
    }
    fn description(&self) -> &'static str {
        "setarch"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_arch::SetarchApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

