use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct FalseApplet;
impl Applet for FalseApplet {
    fn name(&self) -> &'static str {
        "false"
    }
    fn description(&self) -> &'static str {
        "Return an exit code of a failure"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        Ok(1)
    }
}
