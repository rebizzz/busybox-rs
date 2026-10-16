use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct Sha512SumApplet;
impl Applet for Sha512SumApplet {
    fn name(&self) -> &'static str {
        "sha512sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA512 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha512sum", HashType::Sha512, args)
    }
}
