use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SetkeycodesApplet;

impl Applet for SetkeycodesApplet {
    fn name(&self) -> &'static str {
        "setkeycodes"
    }
    fn description(&self) -> &'static str {
        "setkeycodes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::SetkeycodesApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

