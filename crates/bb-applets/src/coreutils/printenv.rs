use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PrintenvApplet;

impl Applet for PrintenvApplet {
    fn name(&self) -> &'static str {
        "printenv"
    }
    fn description(&self) -> &'static str {
        "printenv"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::PrintenvApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

