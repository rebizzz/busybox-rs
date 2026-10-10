use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct WhoamiApplet;

impl Applet for WhoamiApplet {
    fn name(&self) -> &'static str {
        "whoami"
    }
    fn description(&self) -> &'static str {
        "whoami"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::core::WhoamiApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

