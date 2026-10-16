use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CmpApplet;

impl Applet for CmpApplet {
    fn name(&self) -> &'static str {
        "cmp"
    }
    fn description(&self) -> &'static str {
        "cmp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::cmp::CmpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

