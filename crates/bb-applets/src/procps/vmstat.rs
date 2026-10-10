use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct VmstatApplet;

impl Applet for VmstatApplet {
    fn name(&self) -> &'static str {
        "vmstat"
    }
    fn description(&self) -> &'static str {
        "vmstat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::VmstatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

