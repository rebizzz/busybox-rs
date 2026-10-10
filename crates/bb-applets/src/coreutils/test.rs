use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DoubleLBracketApplet;

impl Applet for DoubleLBracketApplet {
    fn name(&self) -> &'static str {
        "test_extended"
    }
    fn description(&self) -> &'static str {
        "test_extended"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::shell::interp::DoubleLBracketApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct TestApplet;

impl Applet for TestApplet {
    fn name(&self) -> &'static str {
        "test"
    }
    fn description(&self) -> &'static str {
        "test"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::TestApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct LBracketApplet;

impl Applet for LBracketApplet {
    fn name(&self) -> &'static str {
        "test"
    }
    fn description(&self) -> &'static str {
        "test"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::util_linux::util::LBracketApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

