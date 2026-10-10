use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct Sha1SumApplet;
impl Applet for Sha1SumApplet {
    fn name(&self) -> &'static str {
        "sha1sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA1 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha1sum", HashType::Sha1, args)
    }
}
