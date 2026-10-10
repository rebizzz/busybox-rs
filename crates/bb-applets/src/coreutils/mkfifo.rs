use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct MkfifoApplet;

impl Applet for MkfifoApplet {
    fn name(&self) -> &'static str {
        "mkfifo"
    }
    fn description(&self) -> &'static str {
        "mkfifo"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::MkfifoApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

