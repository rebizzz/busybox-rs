use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};

pub struct ClearApplet;
impl Applet for ClearApplet {
    fn name(&self) -> &'static str {
        "clear"
    }
    fn description(&self) -> &'static str {
        "Clear the terminal screen"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        print!("\x1b[H\x1b[2J\x1b[3J");
        let _ = io::stdout().flush();
        Ok(0)
    }
}
