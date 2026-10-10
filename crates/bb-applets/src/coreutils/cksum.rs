use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CksumApplet;

impl Applet for CksumApplet {
    fn name(&self) -> &'static str {
        "cksum"
    }
    fn description(&self) -> &'static str {
        "cksum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::CksumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Crc32Applet;

impl Applet for Crc32Applet {
    fn name(&self) -> &'static str {
        "crc32"
    }
    fn description(&self) -> &'static str {
        "crc32"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::Crc32Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

