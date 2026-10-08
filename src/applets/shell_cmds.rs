use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, Read};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;

pub struct XargsApplet;
impl Applet for XargsApplet {
    fn name(&self) -> &'static str {
        "xargs"
    }
    fn description(&self) -> &'static str {
        "Build and execute command lines from standard input"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut null_delim = false;
        let mut max_args: Option<usize> = None;
        let mut cmd_parts = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-0" || bytes == b"--null" {
                null_delim = true;
            } else if bytes == b"-n" {
                if i + 1 < args.len() {
                    max_args = args[i + 1].to_string_lossy().parse().ok();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                max_args = arg.to_string_lossy()[2..].parse().ok();
            } else if bytes == b"-r" || bytes == b"--no-run-if-empty" {
                // busybox xargs by default doesn't run if empty
            } else {
                cmd_parts.push(arg.clone());
            }
            i += 1;
        }

        if cmd_parts.is_empty() {
            cmd_parts.push(OsString::from("echo"));
        }

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let mut input_items = Vec::new();

        if null_delim {
            let mut buf = Vec::new();
            while let Ok(n) = stdin_handle.read_until(0, &mut buf) {
                if n == 0 {
                    break;
                }
                if buf.last() == Some(&0) {
                    buf.pop();
                }
                if !buf.is_empty() {
                    use std::os::unix::ffi::OsStringExt;
                    input_items.push(OsString::from_vec(std::mem::take(&mut buf)));
                }
            }
        } else {
            let mut text = String::new();
            let _ = stdin_handle.read_to_string(&mut text);
            for word in text.split_whitespace() {
                input_items.push(OsString::from(word));
            }
        }

        if input_items.is_empty() {
            return Ok(0);
        }

        let chunk_size = max_args.unwrap_or(input_items.len());
        let mut overall_status = 0;

        for chunk in input_items.chunks(chunk_size) {
            let mut full_cmd = Command::new(&cmd_parts[0]);
            for arg in &cmd_parts[1..] {
                full_cmd.arg(arg);
            }
            for item in chunk {
                full_cmd.arg(item);
            }

            match full_cmd.status() {
                Ok(status) => {
                    let code = status.code().unwrap_or(1);
                    if code != 0 {
                        overall_status = code;
                    }
                }
                Err(e) => {
                    eprintln!("xargs: {}: {}", cmd_parts[0].to_string_lossy(), e);
                    return Ok(127);
                }
            }
        }

        Ok(overall_status)
    }
}

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
            let bytes = arg.as_bytes();
            if bytes == b"-c" {
                if i + 1 < args.len() {
                    command_str = Some(args[i + 1].to_string_lossy().to_string());
                    script_args = args[i + 2..].to_vec();
                    break;
                }
            } else if !bytes.starts_with(b"-") {
                script_file = Some(arg.clone());
                script_args = args[i + 1..].to_vec();
                break;
            }
            i += 1;
        }

        if let Some(cmd) = command_str {
            let status = Command::new("/bin/sh")
                .arg("-c")
                .arg(&cmd)
                .args(&script_args)
                .status()?;
            return Ok(status.code().unwrap_or(0));
        }

        if let Some(file) = script_file {
            let status = Command::new("/bin/sh")
                .arg(&file)
                .args(&script_args)
                .status()?;
            return Ok(status.code().unwrap_or(0));
        }

        // Interactive shell
        let status = Command::new("/bin/sh").status()?;
        Ok(status.code().unwrap_or(0))
    }
}
