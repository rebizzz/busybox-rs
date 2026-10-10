use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PwdxApplet;

impl Applet for PwdxApplet {
    fn name(&self) -> &'static str {
        "pwdx"
    }
    fn description(&self) -> &'static str {
        "pwdx"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::PwdxApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

