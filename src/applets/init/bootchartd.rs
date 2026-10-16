#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct BootchartdApplet;
impl Applet for BootchartdApplet {
    fn name(&self) -> &'static str {
        "bootchartd"
    }
    fn description(&self) -> &'static str {
        "Boot chart collector (subset: start/stop sampling /proc)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out(
                    "bootchartd",
                    "start [DIR] | stop [DIR]",
                    "Sample /proc into a bootchart log",
                );
            }
        }
        if args.is_empty() {
            eprintln!("Usage: bootchartd start [DIR] | stop [DIR]");
            return Ok(1);
        }
        let dir = if args.len() > 1 {
            std::path::PathBuf::from(&args[1])
        } else {
            std::path::PathBuf::from("/var/log/bootchart")
        };
        match args[0].as_bytes() {
            b"start" => bootchart_start(&dir),
            b"stop" => bootchart_stop(&dir),
            other => {
                eprintln!(
                    "bootchartd: unknown command '{}'",
                    String::from_utf8_lossy(other)
                );
                Ok(1)
            }
        }
    }
}
