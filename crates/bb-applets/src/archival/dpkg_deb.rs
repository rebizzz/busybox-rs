use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct DpkgDebApplet;

impl Applet for DpkgDebApplet {
    fn name(&self) -> &'static str {
        "dpkg_deb"
    }
    fn description(&self) -> &'static str {
        "dpkg_deb"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::DpkgDebApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

