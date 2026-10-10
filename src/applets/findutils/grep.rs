use super::common::*;
use crate::core::{Applet, Result};

use std::collections::VecDeque;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct GrepApplet;
impl Applet for GrepApplet {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "Search for PATTERN in FILEs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "grep",
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}
