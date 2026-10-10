use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ExpandApplet;

impl Applet for ExpandApplet {
    fn name(&self) -> &'static str {
        "expand"
    }
    fn description(&self) -> &'static str {
        "expand"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::tabs::ExpandApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UnexpandApplet;

impl Applet for UnexpandApplet {
    fn name(&self) -> &'static str {
        "unexpand"
    }
    fn description(&self) -> &'static str {
        "unexpand"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::tabs::UnexpandApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

