use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct CryptpwApplet;

impl Applet for CryptpwApplet {
    fn name(&self) -> &'static str {
        "cryptpw"
    }
    fn description(&self) -> &'static str {
        "cryptpw"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::CryptpwApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct MkpasswdApplet;

impl Applet for MkpasswdApplet {
    fn name(&self) -> &'static str {
        "mkpasswd"
    }
    fn description(&self) -> &'static str {
        "mkpasswd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::loginutils::login::MkpasswdApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

