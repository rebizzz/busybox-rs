use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GrepApplet;

impl Applet for GrepApplet {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "grep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::findutils::grep::GrepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct EgrepApplet;

impl Applet for EgrepApplet {
    fn name(&self) -> &'static str {
        "egrep"
    }
    fn description(&self) -> &'static str {
        "egrep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::findutils::grep::EgrepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct FgrepApplet;

impl Applet for FgrepApplet {
    fn name(&self) -> &'static str {
        "fgrep"
    }
    fn description(&self) -> &'static str {
        "fgrep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::findutils::grep::FgrepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

