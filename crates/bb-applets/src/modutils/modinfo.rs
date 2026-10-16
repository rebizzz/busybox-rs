use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ModinfoApplet;

impl Applet for ModinfoApplet {
    fn name(&self) -> &'static str {
        "modinfo"
    }
    fn description(&self) -> &'static str {
        "modinfo"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::ModinfoApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

