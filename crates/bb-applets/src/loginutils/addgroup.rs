use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AddgroupApplet;

impl Applet for AddgroupApplet {
    fn name(&self) -> &'static str {
        "addgroup"
    }
    fn description(&self) -> &'static str {
        "addgroup"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::AddgroupApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

