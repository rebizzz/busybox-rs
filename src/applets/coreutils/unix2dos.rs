use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{self, BufReader};
use std::path::Path;

pub struct Unix2dosApplet;
impl Applet for Unix2dosApplet {
    fn name(&self) -> &'static str {
        "unix2dos"
    }
    fn description(&self) -> &'static str {
        "Convert LF line endings to CRLF"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), false)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in &files {
            if *f == Path::new("-") {
                convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), false)?;
            } else if convert_file(f, false, "unix2dos")? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
