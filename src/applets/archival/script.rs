use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub struct ScriptApplet;
impl Applet for ScriptApplet {
    fn name(&self) -> &'static str {
        "script"
    }
    fn description(&self) -> &'static str {
        "Run command logging session to typescript file (pty-less, timestamped)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut append = false;
        let mut file: Option<PathBuf> = None;
        let mut cmd: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-a" || b == b"--append" {
                append = true;
            } else if b == b"-c" && i + 1 < args.len() {
                cmd.push(args[i + 1].clone());
                i += 1;
            } else if !b.starts_with(b"-") {
                if file.is_none() {
                    file = Some(PathBuf::from(&args[i]));
                } else {
                    cmd.push(args[i].clone());
                }
            }
            i += 1;
        }
        let tspath = file.unwrap_or_else(|| PathBuf::from("typescript"));
        let shell_cmd = if cmd.is_empty() {
            None
        } else {
            Some(
                cmd.iter()
                    .map(|c| c.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        };
        let mut ts = match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(&tspath)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("script: {}: {e}", tspath.display());
                return Ok(1);
            }
        };
        let start = std::time::SystemTime::now();
        let _ = writeln!(ts, "Script started at {start:?}");
        let child = if let Some(sh) = shell_cmd {
            Command::new("/bin/sh").arg("-c").arg(sh).spawn()
        } else {
            Command::new("/bin/sh").spawn()
        };
        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!("script: cannot run shell: {e}");
                return Ok(1);
            }
        };

        match child.wait() {
            Ok(st) => {
                let code = st.code().unwrap_or(0);
                let _ = writeln!(
                    ts,
                    "Script done at {:?}, exit={code}",
                    std::time::SystemTime::now()
                );
                Ok(code)
            }
            Err(e) => {
                eprintln!("script: wait: {e}");
                Ok(1)
            }
        }
    }
}
