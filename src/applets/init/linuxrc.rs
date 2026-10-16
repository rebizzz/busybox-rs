#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct LinuxrcApplet;
impl Applet for LinuxrcApplet {
    fn name(&self) -> &'static str {
        "linuxrc"
    }
    fn description(&self) -> &'static str {
        "Init alias used as /linuxrc (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        init_run("linuxrc", args)
    }
}
