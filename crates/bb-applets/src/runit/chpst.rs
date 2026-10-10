use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct EnvdirApplet;

impl Applet for EnvdirApplet {
    fn name(&self) -> &'static str {
        "envdir"
    }
    fn description(&self) -> &'static str {
        "envdir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::EnvdirApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct EnvuidgidApplet;

impl Applet for EnvuidgidApplet {
    fn name(&self) -> &'static str {
        "envuidgid"
    }
    fn description(&self) -> &'static str {
        "envuidgid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::EnvuidgidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SoftlimitApplet;

impl Applet for SoftlimitApplet {
    fn name(&self) -> &'static str {
        "softlimit"
    }
    fn description(&self) -> &'static str {
        "softlimit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::SoftlimitApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct SetuidgidApplet;

impl Applet for SetuidgidApplet {
    fn name(&self) -> &'static str {
        "setuidgid"
    }
    fn description(&self) -> &'static str {
        "setuidgid"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::SetuidgidApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct ChpstApplet;

impl Applet for ChpstApplet {
    fn name(&self) -> &'static str {
        "chpst"
    }
    fn description(&self) -> &'static str {
        "chpst"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::ChpstApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

