#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct PoweroffApplet;
impl Applet for PoweroffApplet {
    fn name(&self) -> &'static str {
        "poweroff"
    }
    fn description(&self) -> &'static str {
        "Halt and power off the system"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        reboot_run("poweroff", libc::LINUX_REBOOT_CMD_POWER_OFF, false, args)
    }
}
