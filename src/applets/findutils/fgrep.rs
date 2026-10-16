use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct FgrepApplet;
impl Applet for FgrepApplet {
    fn name(&self) -> &'static str {
        "fgrep"
    }
    fn description(&self) -> &'static str {
        "Alias to grep -F"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "fgrep",
            fixed_strings: true,
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}
