use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};

pub struct ResetApplet;
impl Applet for ResetApplet {
    fn name(&self) -> &'static str {
        "reset"
    }
    fn description(&self) -> &'static str {
        "Reset the terminal"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        print!("\x1bc");
        let _ = io::stdout().flush();
        Ok(0)
    }
}
