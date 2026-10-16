use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsscsiApplet;

impl Applet for LsscsiApplet {
    fn name(&self) -> &'static str {
        "lsscsi"
    }
    fn description(&self) -> &'static str {
        "lsscsi"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::LsscsiApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

