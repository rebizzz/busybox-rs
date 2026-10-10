use super::common::*;
use crate::core::{Applet, Result};

use std::ffi::OsString;

pub struct GrepApplet;
impl Applet for GrepApplet {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "Search for PATTERN in FILEs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "grep",
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}
