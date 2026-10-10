use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsofApplet;

impl Applet for LsofApplet {
    fn name(&self) -> &'static str {
        "lsof"
    }
    fn description(&self) -> &'static str {
        "lsof"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::LsofApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

