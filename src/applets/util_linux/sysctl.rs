use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct SysctlApplet;
impl Applet for SysctlApplet {
    fn name(&self) -> &'static str {
        "sysctl"
    }
    fn description(&self) -> &'static str {
        "Configure kernel parameters at runtime"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut write_mode = false;
        let mut quiet = false;
        let mut targets = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" || b == b"-A" {
                show_all = true;
            } else if b == b"-w" {
                write_mode = true;
            } else if b == b"-n" || b == b"-q" {
                quiet = true;
            } else if b == b"-p" {
                let conf_path = if i + 1 < args.len() && !args[i + 1].as_bytes().starts_with(b"-") {
                    i += 1;
                    PathBuf::from(&args[i])
                } else {
                    PathBuf::from("/etc/sysctl.conf")
                };
                if let Ok(file) = File::open(&conf_path) {
                    for line in BufReader::new(file)
                        .lines()
                        .map_while(std::result::Result::ok)
                    {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once('=') {
                            let k = k.trim();
                            let v = v.trim();
                            write_sysctl(k, v, quiet);
                        }
                    }
                }
                return Ok(0);
            } else if !b.starts_with(b"-") {
                targets.push(args[i].to_string_lossy().to_string());
            }
            i += 1;
        }

        if show_all {
            dump_sysctl_dir(Path::new("/proc/sys"), "");
            return Ok(0);
        }

        if targets.is_empty() {
            eprintln!("sysctl: no variables specified");
            return Ok(1);
        }

        let mut status = 0;
        for target in targets {
            if target.contains('=') || write_mode {
                let (k, v) = match target.split_once('=') {
                    Some((k, v)) => (k.trim(), v.trim()),
                    None => {
                        eprintln!("sysctl: key=val required for -w");
                        status = 1;
                        continue;
                    }
                };
                if !write_sysctl(k, v, quiet) {
                    status = 1;
                }
            } else {
                let proc_path = sysctl_to_proc(&target);
                match fs::read_to_string(&proc_path) {
                    Ok(val) => {
                        let val = val.trim();
                        if quiet {
                            println!("{}", val);
                        } else {
                            println!("{} = {}", target, val);
                        }
                    }
                    Err(e) => {
                        eprintln!("sysctl: error reading key '{}': {}", target, e);
                        status = 1;
                    }
                }
            }
        }

        Ok(status)
    }
}

fn sysctl_to_proc(key: &str) -> PathBuf {
    let sub = key.replace('.', "/");
    Path::new("/proc/sys").join(sub)
}

fn write_sysctl(key: &str, val: &str, quiet: bool) -> bool {
    let proc_path = sysctl_to_proc(key);
    match fs::write(&proc_path, format!("{}\n", val)) {
        Ok(_) => {
            if !quiet {
                println!("{} = {}", key, val);
            }
            true
        }
        Err(e) => {
            eprintln!("sysctl: setting key '{}': {}", key, e);
            false
        }
    }
}

fn dump_sysctl_dir(dir: &Path, prefix: &str) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let key = if prefix.is_empty() {
                name
            } else {
                format!("{}.{}", prefix, name)
            };
            if path.is_dir() {
                dump_sysctl_dir(&path, &key);
            } else if path.is_file() {
                if let Ok(val) = fs::read_to_string(&path) {
                    let val = val.trim();
                    println!("{} = {}", key, val);
                }
            }
        }
    }
}
