use crate::core::{Applet, Result};
use crate::applets::shell::common::run_shell;
use std::ffi::OsString;

pub struct AshApplet;
impl Applet for AshApplet {
    fn name(&self) -> &'static str {
        "ash"
    }
    fn description(&self) -> &'static str {
        "Command language interpreter (ash)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_shell("ash", args)
    }
}
