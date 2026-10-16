use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LspciApplet;

impl Applet for LspciApplet {
    fn name(&self) -> &'static str {
        "lspci"
    }
    fn description(&self) -> &'static str {
        "lspci"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::LspciApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

