use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UsersApplet;

impl Applet for UsersApplet {
    fn name(&self) -> &'static str {
        "users"
    }
    fn description(&self) -> &'static str {
        "users"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::UsersApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct WhoApplet;

impl Applet for WhoApplet {
    fn name(&self) -> &'static str {
        "who"
    }
    fn description(&self) -> &'static str {
        "who"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::WhoApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct WApplet;

impl Applet for WApplet {
    fn name(&self) -> &'static str {
        "w"
    }
    fn description(&self) -> &'static str {
        "w"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::process_misc::WApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

