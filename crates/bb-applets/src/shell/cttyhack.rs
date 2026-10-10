use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CttyhackApplet;

impl Applet for CttyhackApplet {
    fn name(&self) -> &'static str {
        "cttyhack"
    }
    fn description(&self) -> &'static str {
        "cttyhack"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::CttyhackApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

