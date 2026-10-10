use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct Bzip2Applet;
impl Applet for Bzip2Applet {
    fn name(&self) -> &'static str {
        "bzip2"
    }
    fn description(&self) -> &'static str {
        "Compress or decompress .bz2 files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "bzip2", ".bz2")
    }
}
