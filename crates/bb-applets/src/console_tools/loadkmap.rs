use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LoadkmapApplet;

impl Applet for LoadkmapApplet {
    fn name(&self) -> &'static str {
        "loadkmap"
    }
    fn description(&self) -> &'static str {
        "loadkmap"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::LoadkmapApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

