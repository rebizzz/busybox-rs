use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct UnxzApplet;
impl Applet for UnxzApplet {
    fn name(&self) -> &'static str {
        "unxz"
    }
    fn description(&self) -> &'static str {
        "Decompress .xz files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_unpacker(args, "unxz", ".xz")
    }
}
