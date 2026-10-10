use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MesgApplet;

impl Applet for MesgApplet {
    fn name(&self) -> &'static str {
        "mesg"
    }
    fn description(&self) -> &'static str {
        "mesg"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::MesgApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

