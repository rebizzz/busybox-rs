use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{self, BufReader};
use std::path::Path;

pub struct Dos2unixApplet;
impl Applet for Dos2unixApplet {
    fn name(&self) -> &'static str {
        "dos2unix"
    }
    fn description(&self) -> &'static str {
        "Convert CRLF line endings to LF"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), true)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in &files {
            if *f == Path::new("-") {
                convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), true)?;
            } else if convert_file(f, true, "dos2unix")? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
