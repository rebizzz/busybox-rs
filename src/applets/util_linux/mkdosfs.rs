use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;

pub struct MkdosfsApplet;

impl Applet for MkdosfsApplet {
    fn name(&self) -> &'static str {
        "mkdosfs"
    }
    fn description(&self) -> &'static str {
        "Create an MS-DOS filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_fat_fs(args)
    }
}
