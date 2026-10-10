use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct XxdApplet;

impl Applet for XxdApplet {
    fn name(&self) -> &'static str {
        "xxd"
    }
    fn description(&self) -> &'static str {
        "xxd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::xxd::XxdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

