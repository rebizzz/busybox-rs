use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct NologinApplet;

impl Applet for NologinApplet {
    fn name(&self) -> &'static str {
        "nologin"
    }
    fn description(&self) -> &'static str {
        "nologin"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::NologinApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

