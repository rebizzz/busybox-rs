use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct OdApplet;

impl Applet for OdApplet {
    fn name(&self) -> &'static str {
        "od"
    }
    fn description(&self) -> &'static str {
        "od"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text2::OdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

