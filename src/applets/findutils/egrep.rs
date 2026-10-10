use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct EgrepApplet;
impl Applet for EgrepApplet {
    fn name(&self) -> &'static str {
        "egrep"
    }
    fn description(&self) -> &'static str {
        "Alias to grep -E"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "egrep",
            extended_regex: true,
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}
