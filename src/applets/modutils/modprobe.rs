use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

pub struct ModprobeApplet;

impl Applet for ModprobeApplet {
    fn name(&self) -> &'static str {
        "modprobe"
    }
    fn description(&self) -> &'static str {
        "Add and remove modules from the Linux Kernel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut remove = false;
        let mut target: Option<String> = None;
        let mut opts = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-r" || b == b"--remove" {
                remove = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if target.is_none() {
                target = Some(String::from_utf8_lossy(b).into_owned());
            } else {
                opts.push(arg.clone());
            }
        }

        let module = match target {
            Some(m) => m,
            None => {
                eprintln!("modprobe: module name required");
                return Ok(1);
            }
        };

        let rel = get_kernel_release();
        let dep_file = PathBuf::from(format!("/lib/modules/{}/modules.dep", rel));

        if remove {
            let app = super::rmmod::RmmodApplet;
            return app.run(&[OsString::from(module)]);
        }

        let deps = parse_modules_dep(&dep_file);

        let norm_module = module.replace('-', "_");

        let mut to_load = Vec::new();
        let mut found_path: Option<String> = None;

        for (k, v) in &deps {
            let base = Path::new(k)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace('-', "_");
            if base == norm_module {
                found_path = Some(k.clone());
                for dep in v {
                    to_load.push(dep.clone());
                }
                break;
            }
        }

        if let Some(p) = found_path {
            to_load.reverse();
            let base_dir = PathBuf::from(format!("/lib/modules/{}", rel));
            for dep in to_load {
                let full_path = if dep.starts_with('/') {
                    PathBuf::from(dep)
                } else {
                    base_dir.join(dep)
                };
                let ins = super::insmod::InsmodApplet;
                let _ = ins.run(&[full_path.into_os_string()]);
            }

            let full_path = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                base_dir.join(p)
            };
            let mut ins_args = vec![full_path.into_os_string()];
            ins_args.extend(opts);
            let ins = super::insmod::InsmodApplet;
            ins.run(&ins_args)
        } else {
            let p = Path::new(&module);
            if p.exists() {
                let mut ins_args = vec![p.as_os_str().to_os_string()];
                ins_args.extend(opts);
                let ins = super::insmod::InsmodApplet;
                return ins.run(&ins_args);
            }
            eprintln!(
                "modprobe: module '{}' not found in /lib/modules/{}",
                module, rel
            );
            Ok(1)
        }
    }
}

fn parse_modules_dep(path: &Path) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let f = match File::open(path) {
        Ok(f) => f,
        Err(_) => return map,
    };
    let reader = io::BufReader::new(f);
    for line in reader.lines().map_while(|l| l.ok()) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((mod_part, dep_part)) = trimmed.split_once(':') {
            let mod_path = mod_part.trim().to_string();
            let deps: Vec<String> = dep_part.split_whitespace().map(|s| s.to_string()).collect();
            map.insert(mod_path, deps);
        }
    }
    map
}
