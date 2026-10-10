use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsmodApplet;

impl Applet for LsmodApplet {
    fn name(&self) -> &'static str {
        "lsmod"
    }
    fn description(&self) -> &'static str {
        "lsmod"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::LsmodApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

