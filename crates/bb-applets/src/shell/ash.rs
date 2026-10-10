use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AshApplet;

impl Applet for AshApplet {
    fn name(&self) -> &'static str {
        "ash"
    }
    fn description(&self) -> &'static str {
        "ash"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::AshApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct ShApplet;

impl Applet for ShApplet {
    fn name(&self) -> &'static str {
        "sh"
    }
    fn description(&self) -> &'static str {
        "sh"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::shell::ShApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

