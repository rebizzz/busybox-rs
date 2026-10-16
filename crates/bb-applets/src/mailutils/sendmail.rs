use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct SendmailApplet;

impl Applet for SendmailApplet {
    fn name(&self) -> &'static str {
        "sendmail"
    }
    fn description(&self) -> &'static str {
        "sendmail"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::networking::tools::SendmailApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

