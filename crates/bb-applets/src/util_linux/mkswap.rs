use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MkswapApplet;

impl Applet for MkswapApplet {
    fn name(&self) -> &'static str {
        "mkswap"
    }
    fn description(&self) -> &'static str {
        "mkswap"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::sys_control::MkswapApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

