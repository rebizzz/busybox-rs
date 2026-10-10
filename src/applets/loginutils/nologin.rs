use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct NologinApplet;
impl Applet for NologinApplet {
    fn name(&self) -> &'static str {
        "nologin"
    }
    fn description(&self) -> &'static str {
        "Refuse login (politely)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if let Some(a) = args.first() {
            if is_help(a) {
                return help_out("nologin", "", "Print the nologin message and exit 1");
            }
            eprintln!(
                "nologin: invalid option '{}'",
                String::from_utf8_lossy(a.as_bytes())
            );
            return Ok(1);
        }
        let txt = acct_path("BB_NOLOGIN_TXT", "/etc/nologin.txt");
        match fs::read(&txt) {
            Ok(d) if !d.is_empty() => {
                let stdout = io::stdout();
                let mut o = stdout.lock();
                o.write_all(&d)?;
                if !d.ends_with(b"\n") {
                    o.write_all(b"\n")?;
                }
            }
            _ => {
                eprintln!("This account is currently not available.");
            }
        }
        Ok(1)
    }
}
