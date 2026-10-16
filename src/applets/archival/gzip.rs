use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct GzipApplet;
impl Applet for GzipApplet {
    fn name(&self) -> &'static str {
        "gzip"
    }
    fn description(&self) -> &'static str {
        "Copy input to output (stored only; no zlib)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut to_stdout = false;
        let mut files: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-c" || b == b"--stdout" || b == b"--to-stdout" {
                to_stdout = true;
            } else if b.starts_with(b"-") && b.len() > 1 && !b.starts_with(b"--") {
                if b.iter()
                    .all(|&c| matches!(c, b'c' | b'f' | b'k' | b'v' | b'1'..=b'9' | b'd'))
                {
                    if b.contains(&b'c') {
                        to_stdout = true;
                    }
                } else {
                    files.push(Path::new(a));
                }
            } else {
                files.push(Path::new(a));
            }
        }
        eprintln!("gzip: no zlib in this build; copying without compression");
        if files.is_empty() {
            let mut so = std::io::stdout().lock();
            gzip_copy(Path::new("-"), &mut so)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            let dest: PathBuf = if to_stdout {
                PathBuf::from("-")
            } else {
                let mut s = f.as_os_str().as_bytes().to_vec();
                s.extend_from_slice(b".gz");
                PathBuf::from(std::ffi::OsStr::from_bytes(&s))
            };
            let res: Result<()> = (|| {
                if dest.as_os_str() == "-" {
                    let mut so = std::io::stdout().lock();
                    gzip_copy(f, &mut so)?;
                } else if let Err(e) = (|| -> Result<()> {
                    let mut o = File::create(&dest)?;
                    let _ = gzip_copy(f, &mut o)?;
                    Ok(())
                })() {
                    eprintln!("gzip: {}: {}", f.display(), e);
                    rc = 1;
                }
                Ok(())
            })();
            let _ = res;
        }
        Ok(rc)
    }
}
