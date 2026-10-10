use crate::applets::archival::common::*;
use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct GunzipApplet;
impl Applet for GunzipApplet {
    fn name(&self) -> &'static str {
        "gunzip"
    }
    fn description(&self) -> &'static str {
        "Copy input to output (no zlib inflate)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut to_stdout = false;
        let mut files: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-c" || b == b"--stdout" || b == b"--to-stdout" {
                to_stdout = true;
            } else if b.starts_with(b"-") && !b.starts_with(b"--") && b.len() > 1 {
                if b.contains(&b'c') {
                    to_stdout = true;
                }
            } else {
                files.push(Path::new(a));
            }
        }
        if files.is_empty() {
            let mut so = std::io::stdout().lock();
            let (_, is_gz) = gzip_copy(Path::new("-"), &mut so)?;
            if is_gz {
                eprintln!("gunzip: compressed data requires zlib (not compiled in)");
                return Ok(1);
            }
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            let dest: PathBuf = if to_stdout {
                PathBuf::from("-")
            } else {
                let raw = f.as_os_str().as_bytes();
                let s: &[u8] = raw
                    .strip_suffix(b".gz")
                    .or_else(|| raw.strip_suffix(b".tgz"))
                    .or_else(|| raw.strip_suffix(b".Z"))
                    .unwrap_or(raw);
                PathBuf::from(std::ffi::OsStr::from_bytes(s))
            };
            let r: Result<(u64, bool)> = (|| {
                if dest.as_os_str() == "-" {
                    let mut so = std::io::stdout().lock();
                    Ok(gzip_copy(f, &mut so)?)
                } else {
                    let mut o = File::create(&dest)?;
                    Ok(gzip_copy(f, &mut o)?)
                }
            })();
            match r {
                Ok((_, true)) => {
                    eprintln!(
                        "gunzip: {}: compressed data requires zlib (not compiled in)",
                        f.display()
                    );
                    rc = 1;
                }
                Err(e) => {
                    eprintln!("gunzip: {}: {}", f.display(), e);
                    rc = 1;
                }
                Ok(_) => {}
            }
        }
        Ok(rc)
    }
}

pub struct UncompressAlias;
impl Applet for UncompressAlias {
    fn name(&self) -> &'static str {
        "uncompress"
    }
    fn description(&self) -> &'static str {
        "Alias for gunzip (no zlib inflate)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        GunzipApplet.run(args)
    }
}

fn run_unpacker(args: &[OsString], name: &str, ext: &str) -> Result<i32> {
    let mut to_stdout = name.ends_with("cat");
    let mut files: Vec<&Path> = Vec::new();
    for a in args {
        let b = a.as_bytes();
        if b == b"-c" || b == b"--stdout" || b == b"--to-stdout" {
            to_stdout = true;
        } else if b.starts_with(b"-") && b.len() > 1 && !b.starts_with(b"--") {
            if b.contains(&b'c') {
                to_stdout = true;
            }
        } else {
            files.push(Path::new(a));
        }
    }
    if files.is_empty() {
        let mut so = std::io::stdout().lock();
        let mut inp = open_or_stdin(Path::new("-"))?;
        let _ = copy_stream(&mut inp, &mut so)?;
        return Ok(0);
    }
    let mut rc = 0;
    for f in files {
        if f.as_os_str() == "-" || to_stdout {
            let mut so = std::io::stdout().lock();
            let mut inp = match open_or_stdin(f) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}: {}", name, f.display(), e);
                    rc = 1;
                    continue;
                }
            };
            if let Err(e) = copy_stream(&mut inp, &mut so) {
                eprintln!("{}: {}: {}", name, f.display(), e);
                rc = 1;
            }
        } else {
            let out_name = if let Some(stripped) = f.to_string_lossy().strip_suffix(ext) {
                std::path::PathBuf::from(stripped)
            } else {
                let mut p = f.as_os_str().to_os_string();
                p.push(".out");
                std::path::PathBuf::from(p)
            };
            let mut inp = match open_or_stdin(f) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}: {}", name, f.display(), e);
                    rc = 1;
                    continue;
                }
            };
            let mut out = match File::create(&out_name) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("{}: {}: {}", name, out_name.display(), e);
                    rc = 1;
                    continue;
                }
            };
            if let Err(e) = copy_stream(&mut inp, &mut out) {
                eprintln!("{}: {}: {}", name, f.display(), e);
                rc = 1;
            }
        }
    }
    Ok(rc)
}
