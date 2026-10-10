use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AdduserApplet;

impl Applet for AdduserApplet {
    fn name(&self) -> &'static str {
        "adduser"
    }
    fn description(&self) -> &'static str {
        "adduser"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::AdduserApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

