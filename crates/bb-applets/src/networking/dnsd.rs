use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DnsdApplet;

impl Applet for DnsdApplet {
    fn name(&self) -> &'static str {
        "dnsd"
    }
    fn description(&self) -> &'static str {
        "dnsd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::DnsdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

