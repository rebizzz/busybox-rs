use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ArchApplet;

impl Applet for ArchApplet {
    fn name(&self) -> &'static str {
        "arch"
    }
    fn description(&self) -> &'static str {
        "arch"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::ArchApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UnameApplet;

impl Applet for UnameApplet {
    fn name(&self) -> &'static str {
        "uname"
    }
    fn description(&self) -> &'static str {
        "uname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::UnameApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

