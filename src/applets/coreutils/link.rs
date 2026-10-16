use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::path::Path;

pub struct LinkApplet;
impl Applet for LinkApplet {
    fn name(&self) -> &'static str {
        "link"
    }
    fn description(&self) -> &'static str {
        "Create a link to FILE with the name LINK_NAME"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 2 {
            eprintln!("link: expected 2 arguments");
            return Ok(1);
        }
        fs::hard_link(Path::new(&args[0]), Path::new(&args[1]))?;
        Ok(0)
    }
}
