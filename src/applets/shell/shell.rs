use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::process::Command;

pub struct ShApplet;
impl Applet for ShApplet {
    fn name(&self) -> &'static str {
        "sh"
    }
    fn description(&self) -> &'static str {
        "Command language interpreter (shell)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut command_str: Option<String> = None;
        let mut script_file: Option<OsString> = None;
        let mut script_args: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg == "-c" {
                if i + 1 < args.len() {
                    command_str = Some(args[i + 1].to_string_lossy().to_string());
                    i += 2;
                    continue;
                }
            } else if script_file.is_none() && !arg.to_string_lossy().starts_with('-') {
                script_file = Some(arg.clone());
            } else {
                script_args.push(arg.clone());
            }
            i += 1;
        }

        if let Some(cmd) = command_str {
            let status = Command::new("/bin/sh")
                .arg("-c")
                .arg(cmd)
                .status()?;
            return Ok(status.code().unwrap_or(1));
        }

        if let Some(script) = script_file {
            let status = Command::new("/bin/sh")
                .arg(script)
                .args(script_args)
                .status()?;
            return Ok(status.code().unwrap_or(1));
        }

        let status = Command::new("/bin/sh").status()?;
        Ok(status.code().unwrap_or(0))
    }
}
