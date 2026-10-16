#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct InitApplet;
impl Applet for InitApplet {
    fn name(&self) -> &'static str {
        "init"
    }
    fn description(&self) -> &'static str {
        "Init daemon / runlevel control (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        init_run("init", args)
    }
}
