use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct ArchApplet;
impl Applet for ArchApplet {
    fn name(&self) -> &'static str {
        "arch"
    }
    fn description(&self) -> &'static str {
        "Print machine architecture"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        println!("{}", crate::core::platform::get_machine_arch());
        Ok(0)
    }
}
