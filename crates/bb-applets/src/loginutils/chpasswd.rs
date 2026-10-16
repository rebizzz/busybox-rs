use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChpasswdApplet;

impl Applet for ChpasswdApplet {
    fn name(&self) -> &'static str {
        "chpasswd"
    }
    fn description(&self) -> &'static str {
        "chpasswd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::ChpasswdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

