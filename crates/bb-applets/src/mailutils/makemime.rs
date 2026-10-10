use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MakemimeApplet;

impl Applet for MakemimeApplet {
    fn name(&self) -> &'static str {
        "makemime"
    }
    fn description(&self) -> &'static str {
        "makemime"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::MakemimeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

