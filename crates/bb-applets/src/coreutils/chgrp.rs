use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChgrpApplet;

impl Applet for ChgrpApplet {
    fn name(&self) -> &'static str {
        "chgrp"
    }
    fn description(&self) -> &'static str {
        "chgrp"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::ChgrpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

