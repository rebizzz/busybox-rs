use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct LoadfontApplet;

impl Applet for LoadfontApplet {
    fn name(&self) -> &'static str {
        "loadfont"
    }
    fn description(&self) -> &'static str {
        "loadfont"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::LoadfontApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SetfontApplet;

impl Applet for SetfontApplet {
    fn name(&self) -> &'static str {
        "setfont"
    }
    fn description(&self) -> &'static str {
        "setfont"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::console_tools::console::SetfontApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

