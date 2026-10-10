use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct AddShellApplet;

impl Applet for AddShellApplet {
    fn name(&self) -> &'static str {
        "add_shell"
    }
    fn description(&self) -> &'static str {
        "add_shell"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::AddShellApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct RemoveShellApplet;

impl Applet for RemoveShellApplet {
    fn name(&self) -> &'static str {
        "remove_shell"
    }
    fn description(&self) -> &'static str {
        "remove_shell"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::RemoveShellApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

