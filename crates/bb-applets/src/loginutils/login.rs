use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LoginApplet;

impl Applet for LoginApplet {
    fn name(&self) -> &'static str {
        "login"
    }
    fn description(&self) -> &'static str {
        "login"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::LoginApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

