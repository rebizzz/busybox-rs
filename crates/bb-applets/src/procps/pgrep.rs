use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct PgrepApplet;

impl Applet for PgrepApplet {
    fn name(&self) -> &'static str {
        "pgrep"
    }
    fn description(&self) -> &'static str {
        "pgrep"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::PgrepApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct PkillApplet;

impl Applet for PkillApplet {
    fn name(&self) -> &'static str {
        "pkill"
    }
    fn description(&self) -> &'static str {
        "pkill"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::PkillApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

