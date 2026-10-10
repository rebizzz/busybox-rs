use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PasswdApplet;

impl Applet for PasswdApplet {
    fn name(&self) -> &'static str {
        "passwd"
    }
    fn description(&self) -> &'static str {
        "passwd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::PasswdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

