use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HeadApplet;

impl Applet for HeadApplet {
    fn name(&self) -> &'static str {
        "head"
    }
    fn description(&self) -> &'static str {
        "head"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text::HeadApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

