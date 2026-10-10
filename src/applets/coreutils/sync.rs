use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct SyncApplet;
impl Applet for SyncApplet {
    fn name(&self) -> &'static str {
        "sync"
    }
    fn description(&self) -> &'static str {
        "Force changed blocks to disk"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        crate::core::platform::sync_disks();
        Ok(0)
    }
}
