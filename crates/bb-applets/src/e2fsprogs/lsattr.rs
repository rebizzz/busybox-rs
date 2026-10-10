use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LsattrApplet;

impl Applet for LsattrApplet {
    fn name(&self) -> &'static str {
        "lsattr"
    }
    fn description(&self) -> &'static str {
        "lsattr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::LsattrApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

