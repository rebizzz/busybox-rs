use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct KlogdApplet;

impl Applet for KlogdApplet {
    fn name(&self) -> &'static str {
        "klogd"
    }
    fn description(&self) -> &'static str {
        "klogd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::KlogdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

