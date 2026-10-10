use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct TrueApplet;
impl Applet for TrueApplet {
    fn name(&self) -> &'static str {
        "true"
    }
    fn description(&self) -> &'static str {
        "Return an exit code of success"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        Ok(0)
    }
}
