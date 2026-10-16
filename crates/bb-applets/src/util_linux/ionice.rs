use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct IoniceApplet;

impl Applet for IoniceApplet {
    fn name(&self) -> &'static str {
        "ionice"
    }
    fn description(&self) -> &'static str {
        "ionice"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::procps::procps::IoniceApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

