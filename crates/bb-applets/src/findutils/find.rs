use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FindApplet;

impl Applet for FindApplet {
    fn name(&self) -> &'static str {
        "find"
    }
    fn description(&self) -> &'static str {
        "find"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::findutils::find::FindApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

