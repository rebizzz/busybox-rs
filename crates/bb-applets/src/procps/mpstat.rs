use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MpstatApplet;

impl Applet for MpstatApplet {
    fn name(&self) -> &'static str {
        "mpstat"
    }
    fn description(&self) -> &'static str {
        "mpstat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::MpstatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

