use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct LzmaApplet;
impl Applet for LzmaApplet {
    fn name(&self) -> &'static str {
        "lzma"
    }
    fn description(&self) -> &'static str {
        "Compress or decompress .lzma files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "lzma", ".lzma")
    }
}
