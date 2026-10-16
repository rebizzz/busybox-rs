use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HdApplet;

impl Applet for HdApplet {
    fn name(&self) -> &'static str {
        "hd"
    }
    fn description(&self) -> &'static str {
        "hd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::HdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct HexdumpApplet;

impl Applet for HexdumpApplet {
    fn name(&self) -> &'static str {
        "hexdump"
    }
    fn description(&self) -> &'static str {
        "hexdump"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::HexdumpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

