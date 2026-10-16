use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::path::Path;

pub struct UnlinkApplet;
impl Applet for UnlinkApplet {
    fn name(&self) -> &'static str {
        "unlink"
    }
    fn description(&self) -> &'static str {
        "Call the unlink function to remove the specified FILE"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 1 {
            eprintln!("unlink: expected 1 argument");
            return Ok(1);
        }
        fs::remove_file(Path::new(&args[0]))?;
        Ok(0)
    }
}
