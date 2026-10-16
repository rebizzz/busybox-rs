use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct StringsApplet;

impl Applet for StringsApplet {
    fn name(&self) -> &'static str {
        "strings"
    }
    fn description(&self) -> &'static str {
        "strings"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::stream::StringsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

