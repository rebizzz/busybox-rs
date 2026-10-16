use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Killall5Applet;

impl Applet for Killall5Applet {
    fn name(&self) -> &'static str {
        "killall5"
    }
    fn description(&self) -> &'static str {
        "killall5"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::Killall5Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct KillApplet;

impl Applet for KillApplet {
    fn name(&self) -> &'static str {
        "kill"
    }
    fn description(&self) -> &'static str {
        "kill"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::KillApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct KillallApplet;

impl Applet for KillallApplet {
    fn name(&self) -> &'static str {
        "killall"
    }
    fn description(&self) -> &'static str {
        "killall"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::KillallApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

