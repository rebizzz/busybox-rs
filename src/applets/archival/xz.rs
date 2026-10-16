use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct XzApplet;
impl Applet for XzApplet {
    fn name(&self) -> &'static str {
        "xz"
    }
    fn description(&self) -> &'static str {
        "Compress or decompress .xz files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "xz", ".xz")
    }
}
