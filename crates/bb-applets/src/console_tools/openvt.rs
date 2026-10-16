use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct OpenvtApplet;

impl Applet for OpenvtApplet {
    fn name(&self) -> &'static str {
        "openvt"
    }
    fn description(&self) -> &'static str {
        "openvt"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::OpenvtApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

