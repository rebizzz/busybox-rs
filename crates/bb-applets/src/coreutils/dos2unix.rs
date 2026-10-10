use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Dos2unixApplet;

impl Applet for Dos2unixApplet {
    fn name(&self) -> &'static str {
        "dos2unix"
    }
    fn description(&self) -> &'static str {
        "dos2unix"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text2::Dos2unixApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Unix2dosApplet;

impl Applet for Unix2dosApplet {
    fn name(&self) -> &'static str {
        "unix2dos"
    }
    fn description(&self) -> &'static str {
        "unix2dos"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text2::Unix2dosApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

