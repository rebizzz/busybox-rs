use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PasteApplet;

impl Applet for PasteApplet {
    fn name(&self) -> &'static str {
        "paste"
    }
    fn description(&self) -> &'static str {
        "paste"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text2::PasteApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

