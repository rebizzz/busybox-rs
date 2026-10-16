use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DumpkmapApplet;

impl Applet for DumpkmapApplet {
    fn name(&self) -> &'static str {
        "dumpkmap"
    }
    fn description(&self) -> &'static str {
        "dumpkmap"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::DumpkmapApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

