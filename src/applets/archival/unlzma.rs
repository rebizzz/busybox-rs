use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct UnlzmaApplet;
impl Applet for UnlzmaApplet {
    fn name(&self) -> &'static str {
        "unlzma"
    }
    fn description(&self) -> &'static str {
        "Decompress .lzma files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "unlzma", ".lzma")
    }
}
