use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct LzcatApplet;
impl Applet for LzcatApplet {
    fn name(&self) -> &'static str {
        "lzcat"
    }
    fn description(&self) -> &'static str {
        "Decompress .lzma files to stdout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "lzcat", ".lzma")
    }
}
