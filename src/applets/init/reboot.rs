#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct RebootApplet;
impl Applet for RebootApplet {
    fn name(&self) -> &'static str {
        "reboot"
    }
    fn description(&self) -> &'static str {
        "Reboot the system"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        reboot_run("reboot", libc::LINUX_REBOOT_CMD_RESTART, false, args)
    }
}
