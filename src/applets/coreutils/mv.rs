use crate::core::fs::copy_recursive;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct MvApplet;
impl Applet for MvApplet {
    fn name(&self) -> &'static str {
        "mv"
    }
    fn description(&self) -> &'static str {
        "Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut target_dir: Option<PathBuf> = None;
        let mut targets = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-t" {
                if i + 1 < args.len() {
                    target_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-t") && bytes.len() > 2 {
                target_dir = Some(PathBuf::from(std::ffi::OsStr::from_bytes(&bytes[2..])));
            } else if bytes.starts_with(b"--target-directory=") {
                target_dir = Some(PathBuf::from(std::ffi::OsStr::from_bytes(&bytes[19..])));
            } else if !bytes.starts_with(b"-") || bytes == b"-" {
                targets.push(Path::new(arg));
            }
            i += 1;
        }

        let (sources, dst) = if let Some(ref tdir) = target_dir {
            if targets.is_empty() {
                eprintln!("mv: missing file operand");
                return Ok(1);
            }
            (&targets[..], tdir.as_path())
        } else {
            if targets.len() < 2 {
                eprintln!("mv: missing file operand");
                return Ok(1);
            }
            (&targets[..targets.len() - 1], *targets.last().unwrap())
        };

        let dst_is_dir = fs::metadata(dst).map(|m| m.is_dir()).unwrap_or(false);

        if sources.len() > 1 && !dst_is_dir {
            eprintln!("mv: target '{}' is not a directory", dst.display());
            return Ok(1);
        }

        for src in sources {
            let target_path = if dst_is_dir {
                let name = src.file_name().unwrap_or(src.as_os_str());
                dst.join(name)
            } else {
                dst.to_path_buf()
            };

            if fs::rename(src, &target_path).is_err() {
                copy_recursive(src, &target_path)?;
                if fs::metadata(src).map(|m| m.is_dir()).unwrap_or(false) {
                    fs::remove_dir_all(src)?;
                } else {
                    fs::remove_file(src)?;
                }
            }
        }
        Ok(0)
    }
}
