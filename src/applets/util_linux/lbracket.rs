use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct LBracketApplet;

impl Applet for LBracketApplet {
    fn name(&self) -> &'static str {
        "["
    }
    fn description(&self) -> &'static str {
        "Evaluate a conditional expression (alias for test)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        super::test::run_test("[", args)
    }
}
