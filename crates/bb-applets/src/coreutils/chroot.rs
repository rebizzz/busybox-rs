use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct ChrootApplet;

impl Applet for ChrootApplet {
    fn name(&self) -> &'static str {
        "chroot"
    }
    fn description(&self) -> &'static str {
        "chroot"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::ChrootApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

