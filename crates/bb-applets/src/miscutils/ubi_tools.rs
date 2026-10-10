use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct UbiattachApplet;

impl Applet for UbiattachApplet {
    fn name(&self) -> &'static str {
        "ubiattach"
    }
    fn description(&self) -> &'static str {
        "ubiattach"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbiattachApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UbidetachApplet;

impl Applet for UbidetachApplet {
    fn name(&self) -> &'static str {
        "ubidetach"
    }
    fn description(&self) -> &'static str {
        "ubidetach"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbidetachApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UbimkvolApplet;

impl Applet for UbimkvolApplet {
    fn name(&self) -> &'static str {
        "ubimkvol"
    }
    fn description(&self) -> &'static str {
        "ubimkvol"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbimkvolApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UbirmvolApplet;

impl Applet for UbirmvolApplet {
    fn name(&self) -> &'static str {
        "ubirmvol"
    }
    fn description(&self) -> &'static str {
        "ubirmvol"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbirmvolApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UbirsvolApplet;

impl Applet for UbirsvolApplet {
    fn name(&self) -> &'static str {
        "ubirsvol"
    }
    fn description(&self) -> &'static str {
        "ubirsvol"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbirsvolApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UbiupdatevolApplet;

impl Applet for UbiupdatevolApplet {
    fn name(&self) -> &'static str {
        "ubiupdatevol"
    }
    fn description(&self) -> &'static str {
        "ubiupdatevol"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::UbiupdatevolApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

