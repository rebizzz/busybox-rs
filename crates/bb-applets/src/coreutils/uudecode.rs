use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Base32Applet;

impl Applet for Base32Applet {
    fn name(&self) -> &'static str {
        "base32"
    }
    fn description(&self) -> &'static str {
        "base32"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::Base32Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Base64Applet;

impl Applet for Base64Applet {
    fn name(&self) -> &'static str {
        "base64"
    }
    fn description(&self) -> &'static str {
        "base64"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::Base64Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UudecodeApplet;

impl Applet for UudecodeApplet {
    fn name(&self) -> &'static str {
        "uudecode"
    }
    fn description(&self) -> &'static str {
        "uudecode"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::uudecode::UudecodeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

