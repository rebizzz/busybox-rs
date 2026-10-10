use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

pub struct EnvApplet;

impl Applet for EnvApplet {
    fn name(&self) -> &'static str {
        "env"
    }
    fn description(&self) -> &'static str {
        "Set environment and run command, or print environment"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ignore_env = false;
        let mut idx = 0;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-i" || b == b"-" || b == b"--ignore-environment" {
                ignore_env = true;
            } else if b.starts_with(b"-") {
            } else {
                break;
            }
            idx += 1;
        }

        let mut custom_env: Vec<(OsString, OsString)> = Vec::new();
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if let Some(pos) = b.iter().position(|&c| c == b'=') {
                let k = OsStr::from_bytes(&b[..pos]).to_os_string();
                let v = OsStr::from_bytes(&b[pos + 1..]).to_os_string();
                custom_env.push((k, v));
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            let out = io::stdout();
            let mut lock = out.lock();
            if !ignore_env {
                for (k, v) in std::env::vars_os() {
                    if !custom_env.iter().any(|(ck, _)| ck == &k) {
                        lock.write_all(k.as_bytes())?;
                        lock.write_all(b"=")?;
                        lock.write_all(v.as_bytes())?;
                        lock.write_all(b"\n")?;
                    }
                }
            }
            for (k, v) in custom_env {
                lock.write_all(k.as_bytes())?;
                lock.write_all(b"=")?;
                lock.write_all(v.as_bytes())?;
                lock.write_all(b"\n")?;
            }
            return Ok(0);
        }

        let prog = &args[idx];
        let prog_args = &args[idx + 1..];

        let mut cmd = Command::new(prog);
        cmd.args(prog_args);
        if ignore_env {
            cmd.env_clear();
        }
        for (k, v) in custom_env {
            cmd.env(k, v);
        }

        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("env: {}: {}", Path::new(prog).display(), e);
                Ok(127)
            }
        }
    }
}

