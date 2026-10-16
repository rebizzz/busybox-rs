use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct Bunzip2Applet;
impl Applet for Bunzip2Applet {
    fn name(&self) -> &'static str {
        "bunzip2"
    }
    fn description(&self) -> &'static str {
        "Decompress .bz2 files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "bunzip2", ".bz2")
    }
}
