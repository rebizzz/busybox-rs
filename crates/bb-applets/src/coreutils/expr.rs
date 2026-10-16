use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ExprApplet;

impl Applet for ExprApplet {
    fn name(&self) -> &'static str {
        "expr"
    }
    fn description(&self) -> &'static str {
        "expr"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::editors::editor::ExprApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

