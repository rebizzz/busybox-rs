use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::env;

pub struct PwdApplet;
impl Applet for PwdApplet {
    fn name(&self) -> &'static str {
        "pwd"
    }
    fn description(&self) -> &'static str {
        "Print the current working directory"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let dir = env::current_dir()?;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(dir.as_os_str().as_bytes())?;
        handle.write_all(b"\n")?;
        Ok(0)
    }
}
