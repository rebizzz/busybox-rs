use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct Sha256SumApplet;
impl Applet for Sha256SumApplet {
    fn name(&self) -> &'static str {
        "sha256sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA256 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha256sum", HashType::Sha256, args)
    }
}
