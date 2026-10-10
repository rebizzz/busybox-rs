use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SeqApplet;

impl Applet for SeqApplet {
    fn name(&self) -> &'static str {
        "seq"
    }
    fn description(&self) -> &'static str {
        "seq"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::numbers::SeqApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

