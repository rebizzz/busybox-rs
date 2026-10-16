use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RpmApplet;

impl Applet for RpmApplet {
    fn name(&self) -> &'static str {
        "rpm"
    }
    fn description(&self) -> &'static str {
        "rpm"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::RpmApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Rpm2cpioApplet;

impl Applet for Rpm2cpioApplet {
    fn name(&self) -> &'static str {
        "rpm2cpio"
    }
    fn description(&self) -> &'static str {
        "rpm2cpio"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::Rpm2cpioApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

