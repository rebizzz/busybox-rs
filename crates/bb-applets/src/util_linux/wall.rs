use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WallApplet;

impl Applet for WallApplet {
    fn name(&self) -> &'static str {
        "wall"
    }
    fn description(&self) -> &'static str {
        "wall"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::init::init::WallApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

