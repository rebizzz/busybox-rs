use crate::core::{Applet, Result};
use crate::applets::shell::common::run_shell;
use std::ffi::OsString;

pub struct HushApplet;
impl Applet for HushApplet {
    fn name(&self) -> &'static str {
        "hush"
    }
    fn description(&self) -> &'static str {
        "Command language interpreter (hush)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_shell("hush", args)
    }
}
