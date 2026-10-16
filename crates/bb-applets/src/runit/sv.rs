use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SvApplet;

impl Applet for SvApplet {
    fn name(&self) -> &'static str {
        "sv"
    }
    fn description(&self) -> &'static str {
        "sv"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::SvApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SvcApplet;

impl Applet for SvcApplet {
    fn name(&self) -> &'static str {
        "svc"
    }
    fn description(&self) -> &'static str {
        "svc"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::SvcApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SvokApplet;

impl Applet for SvokApplet {
    fn name(&self) -> &'static str {
        "svok"
    }
    fn description(&self) -> &'static str {
        "svok"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::SvokApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

