use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct HttpdApplet;

impl Applet for HttpdApplet {
    fn name(&self) -> &'static str {
        "httpd"
    }
    fn description(&self) -> &'static str {
        "httpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::HttpdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

