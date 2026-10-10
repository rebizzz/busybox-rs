use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DeluserApplet;

impl Applet for DeluserApplet {
    fn name(&self) -> &'static str {
        "deluser"
    }
    fn description(&self) -> &'static str {
        "deluser"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::DeluserApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct DelgroupApplet;

impl Applet for DelgroupApplet {
    fn name(&self) -> &'static str {
        "delgroup"
    }
    fn description(&self) -> &'static str {
        "delgroup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::DelgroupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

