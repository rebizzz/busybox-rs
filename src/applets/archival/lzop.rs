use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct LzopApplet;
impl Applet for LzopApplet {
    fn name(&self) -> &'static str {
        "lzop"
    }
    fn description(&self) -> &'static str {
        "Compress or decompress .lzo files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "lzop", ".lzo")
    }
}
