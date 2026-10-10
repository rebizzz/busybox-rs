use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SuApplet;

impl Applet for SuApplet {
    fn name(&self) -> &'static str {
        "su"
    }
    fn description(&self) -> &'static str {
        "su"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::SuApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

