use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FuserApplet;

impl Applet for FuserApplet {
    fn name(&self) -> &'static str {
        "fuser"
    }
    fn description(&self) -> &'static str {
        "fuser"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::FuserApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

