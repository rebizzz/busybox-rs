use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HexeditApplet;

impl Applet for HexeditApplet {
    fn name(&self) -> &'static str {
        "hexedit"
    }
    fn description(&self) -> &'static str {
        "hexedit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::HexeditApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

