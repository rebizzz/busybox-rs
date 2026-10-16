use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SortApplet;

impl Applet for SortApplet {
    fn name(&self) -> &'static str {
        "sort"
    }
    fn description(&self) -> &'static str {
        "sort"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::sort::SortApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

