use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub struct DepmodApplet;

impl Applet for DepmodApplet {
    fn name(&self) -> &'static str {
        "depmod"
    }
    fn description(&self) -> &'static str {
        "Generate modules.dep and map files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut rel: Option<String> = None;
        for arg in args {
            let b = arg.as_bytes();
            if !b.is_empty() && b[0] == b'-' {
            } else if rel.is_none() {
                rel = Some(String::from_utf8_lossy(b).into_owned());
            }
        }

        let kversion = rel.unwrap_or_else(get_kernel_release);
        let mod_dir = PathBuf::from(format!("/lib/modules/{}", kversion));
        let dep_file = mod_dir.join("modules.dep");

        if !mod_dir.exists() {
            eprintln!("depmod: directory {} does not exist", mod_dir.display());
            return Ok(1);
        }

        let mut ko_files = Vec::new();
        let mut stack = vec![mod_dir.clone()];
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Some(ext) = path.extension() {
                        if ext == "ko" || path.to_string_lossy().contains(".ko.") {
                            if let Ok(rel_path) = path.strip_prefix(&mod_dir) {
                                ko_files.push(rel_path.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }

        ko_files.sort();

        let mut out = Vec::new();
        for k in &ko_files {
            out.extend_from_slice(k.as_bytes());
            out.extend_from_slice(b":\n");
        }

        if let Err(e) = fs::write(&dep_file, &out) {
            eprintln!("depmod: cannot write {}: {}", dep_file.display(), e);
            return Ok(1);
        }

        Ok(0)
    }
}
