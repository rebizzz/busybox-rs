use crate::core::{Applet, Result};
use crate::applets::miscutils::common::run_pager;
use std::ffi::OsString;

pub struct LessApplet;
impl Applet for LessApplet {
    fn name(&self) -> &'static str {
        "less"
    }
    fn description(&self) -> &'static str {
        "Opposite of more: view file contents with backward scrolling"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_pager(true, args)
    }
}
