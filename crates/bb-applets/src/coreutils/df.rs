use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DfApplet;

impl Applet for DfApplet {
    fn name(&self) -> &'static str {
        "df"
    }
    fn description(&self) -> &'static str {
        "df"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::perms::DfApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

