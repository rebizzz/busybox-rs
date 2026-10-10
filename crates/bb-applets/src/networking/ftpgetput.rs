use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct FtpgetApplet;

impl Applet for FtpgetApplet {
    fn name(&self) -> &'static str {
        "ftpget"
    }
    fn description(&self) -> &'static str {
        "ftpget"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::FtpgetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct FtpputApplet;

impl Applet for FtpputApplet {
    fn name(&self) -> &'static str {
        "ftpput"
    }
    fn description(&self) -> &'static str {
        "ftpput"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::servers::FtpputApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

