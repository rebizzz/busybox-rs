use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct Md5SumApplet;
impl Applet for Md5SumApplet {
    fn name(&self) -> &'static str {
        "md5sum"
    }
    fn description(&self) -> &'static str {
        "Print or check MD5 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("md5sum", HashType::Md5, args)
    }
}
