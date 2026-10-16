use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct TreeApplet;

impl Applet for TreeApplet {
    fn name(&self) -> &'static str {
        "tree"
    }
    fn description(&self) -> &'static str {
        "tree"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::TreeApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

