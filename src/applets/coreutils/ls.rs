use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LsApplet;
impl Applet for LsApplet {
    fn name(&self) -> &'static str {
        "ls"
    }
    fn description(&self) -> &'static str {
        "List directory contents"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut paths = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    if b == b'a' {
                        show_all = true;
                    }
                }
            } else {
                paths.push(Path::new(arg));
            }
        }

        if paths.is_empty() {
            paths.push(Path::new("."));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let multiple = paths.len() > 1;
        let mut exit_code = 0;

        for (i, p) in paths.iter().enumerate() {
            if multiple {
                if i > 0 {
                    handle.write_all(b"\n")?;
                }
                handle.write_all(p.as_os_str().as_bytes())?;
                handle.write_all(b":\n")?;
            }

            match fs::metadata(p) {
                Ok(meta) if meta.is_dir() => {
                    let mut entries = Vec::new();
                    match fs::read_dir(p) {
                        Ok(dir) => {
                            for e in dir.flatten() {
                                let name = e.file_name();
                                if !show_all && name.as_bytes().starts_with(b".") {
                                    continue;
                                }
                                entries.push(name);
                            }
                            entries.sort();
                            for entry in entries {
                                handle.write_all(entry.as_bytes())?;
                                handle.write_all(b"\n")?;
                            }
                        }
                        Err(e) => {
                            eprintln!("ls: {}: {}", p.display(), e);
                            exit_code = 1;
                        }
                    }
                }
                Ok(_) => {
                    handle.write_all(p.as_os_str().as_bytes())?;
                    handle.write_all(b"\n")?;
                }
                Err(e) => {
                    eprintln!("ls: {}: {}", p.display(), e);
                    exit_code = 1;
                }
            }
        }
        Ok(exit_code)
    }
}
