use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FoldApplet;

impl Applet for FoldApplet {
    fn name(&self) -> &'static str {
        "fold"
    }
    fn description(&self) -> &'static str {
        "fold"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::fold::FoldApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

