use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct RxApplet;

impl Applet for RxApplet {
    fn name(&self) -> &'static str {
        "rx"
    }
    fn description(&self) -> &'static str {
        "rx"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::RxApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

