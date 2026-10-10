use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct InstallApplet;

impl Applet for InstallApplet {
    fn name(&self) -> &'static str {
        "install"
    }
    fn description(&self) -> &'static str {
        "Copy files and set attributes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut directory_mode = false;
        let mut mode: Option<u32> = None;
        let mut targets: Vec<&Path> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" || b == b"--directory" {
                directory_mode = true;
            } else if b == b"-m" || b == b"--mode" {
                if i + 1 < args.len() {
                    i += 1;
                    mode = u32::from_str_radix(
                        std::str::from_utf8(args[i].as_bytes()).unwrap_or("755"),
                        8,
                    )
                    .ok();
                }
            } else if b.starts_with(b"-m") && b.len() > 2 {
                mode = u32::from_str_radix(std::str::from_utf8(&b[2..]).unwrap_or("755"), 8).ok();
            } else if b.starts_with(b"-") {
            } else {
                targets.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if directory_mode {
            let dir_mode = mode.unwrap_or(0o755);
            for d in targets {
                if let Err(e) = fs::create_dir_all(d) {
                    eprintln!("install: {}: {}", d.display(), e);
                    return Ok(1);
                }
                let _ = fs::set_permissions(d, fs::Permissions::from_mode(dir_mode));
            }
            return Ok(0);
        }

        if targets.len() < 2 {
            eprintln!("install: missing destination file operand");
            return Ok(1);
        }

        let file_mode = mode.unwrap_or(0o755);
        let dest = targets.last().unwrap();
        let sources = &targets[..targets.len() - 1];

        if dest.is_dir() {
            for src in sources {
                let fname = match src.file_name() {
                    Some(f) => f,
                    None => continue,
                };
                let target_path = dest.join(fname);
                if let Err(e) = fs::copy(src, &target_path) {
                    eprintln!("install: {}: {}", src.display(), e);
                    return Ok(1);
                }
                let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(file_mode));
            }
        } else {
            if sources.len() > 1 {
                eprintln!("install: target '{}' is not a directory", dest.display());
                return Ok(1);
            }
            if let Err(e) = fs::copy(sources[0], dest) {
                eprintln!("install: {}: {}", sources[0].display(), e);
                return Ok(1);
            }
            let _ = fs::set_permissions(dest, fs::Permissions::from_mode(file_mode));
        }

        Ok(0)
    }
}

