use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MimApplet;

impl Applet for MimApplet {
    fn name(&self) -> &'static str {
        "mim"
    }
    fn description(&self) -> &'static str {
        "mim"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::MimApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

