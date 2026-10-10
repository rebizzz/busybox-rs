use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AcpidApplet;

impl Applet for AcpidApplet {
    fn name(&self) -> &'static str {
        "acpid"
    }
    fn description(&self) -> &'static str {
        "acpid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::modutils::modules::AcpidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

