use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct InsmodApplet;

impl Applet for InsmodApplet {
    fn name(&self) -> &'static str {
        "insmod"
    }
    fn description(&self) -> &'static str {
        "insmod"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::InsmodApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

