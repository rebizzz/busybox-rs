use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct XzcatApplet;
impl Applet for XzcatApplet {
    fn name(&self) -> &'static str {
        "xzcat"
    }
    fn description(&self) -> &'static str {
        "Decompress .xz files to stdout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "xzcat", ".xz")
    }
}
