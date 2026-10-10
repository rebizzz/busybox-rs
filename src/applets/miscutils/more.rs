use crate::core::{Applet, Result};
use crate::applets::miscutils::common::run_pager;
use std::ffi::OsString;

pub struct MoreApplet;
impl Applet for MoreApplet {
    fn name(&self) -> &'static str {
        "more"
    }
    fn description(&self) -> &'static str {
        "View file contents one screenful at a time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_pager(false, args)
    }
}
