use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct BzcatApplet;
impl Applet for BzcatApplet {
    fn name(&self) -> &'static str {
        "bzcat"
    }
    fn description(&self) -> &'static str {
        "Decompress .bz2 files to stdout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "bzcat", ".bz2")
    }
}
