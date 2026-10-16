use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CrondApplet;

impl Applet for CrondApplet {
    fn name(&self) -> &'static str {
        "crond"
    }
    fn description(&self) -> &'static str {
        "crond"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::CrondApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

