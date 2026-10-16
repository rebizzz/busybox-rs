use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SuloginApplet;

impl Applet for SuloginApplet {
    fn name(&self) -> &'static str {
        "sulogin"
    }
    fn description(&self) -> &'static str {
        "sulogin"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::SuloginApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

