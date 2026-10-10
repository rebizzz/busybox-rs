use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IpruleApplet;
impl Applet for IpruleApplet {
    fn name(&self) -> &'static str {
        "iprule"
    }
    fn description(&self) -> &'static str {
        "Manage routing policy database"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_rule(args)
    }
}
