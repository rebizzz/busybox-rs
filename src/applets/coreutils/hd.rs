use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct HdApplet;
impl Applet for HdApplet {
    fn name(&self) -> &'static str {
        "hd"
    }
    fn description(&self) -> &'static str {
        "Hexdump in canonical form"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        super::hexdump::run_hd(args)
    }
}
