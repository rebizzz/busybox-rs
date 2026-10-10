use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FakeidentdApplet;

impl Applet for FakeidentdApplet {
    fn name(&self) -> &'static str {
        "fakeidentd"
    }
    fn description(&self) -> &'static str {
        "fakeidentd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::FakeidentdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

