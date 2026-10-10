use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct Mke2fsApplet;

impl Applet for Mke2fsApplet {
    fn name(&self) -> &'static str {
        "mke2fs"
    }
    fn description(&self) -> &'static str {
        "Create an ext2/ext3/ext4 filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_ext2_fs(args)
    }
}
