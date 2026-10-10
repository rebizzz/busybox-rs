use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IdApplet;

impl Applet for IdApplet {
    fn name(&self) -> &'static str {
        "id"
    }
    fn description(&self) -> &'static str {
        "id"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::IdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct GroupsApplet;

impl Applet for GroupsApplet {
    fn name(&self) -> &'static str {
        "groups"
    }
    fn description(&self) -> &'static str {
        "groups"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::sysinfo::GroupsApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

