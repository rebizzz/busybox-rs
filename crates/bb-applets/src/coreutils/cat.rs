use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CatApplet;

impl Applet for CatApplet {
    fn name(&self) -> &'static str {
        "cat"
    }
    fn description(&self) -> &'static str {
        "cat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::text::CatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

