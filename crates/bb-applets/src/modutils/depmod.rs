use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DepmodApplet;

impl Applet for DepmodApplet {
    fn name(&self) -> &'static str {
        "depmod"
    }
    fn description(&self) -> &'static str {
        "depmod"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::DepmodApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

