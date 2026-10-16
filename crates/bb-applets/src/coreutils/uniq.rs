use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UniqApplet;

impl Applet for UniqApplet {
    fn name(&self) -> &'static str {
        "uniq"
    }
    fn description(&self) -> &'static str {
        "uniq"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::stream::UniqApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

