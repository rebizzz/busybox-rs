use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

use crate::applets::archival::dpkg_deb::*;
fn dpkg_status_db() -> PathBuf {
    std::env::var_os("DPKG_ADMINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/var/lib/dpkg"))
        .join("status")
}

pub struct DpkgApplet;
impl Applet for DpkgApplet {
    fn name(&self) -> &'static str {
        "dpkg"
    }
    fn description(&self) -> &'static str {
        "Deb package info/install subset (-l/-s list, -i/--unpack install)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let a: Vec<&[u8]> = args.iter().map(ab).collect();
        if a.is_empty() {
            eprintln!("dpkg: need -l/-s/-L/-i/--unpack");
            return Ok(1);
        }
        let act = a[0];
        if act == b"-l" || act == b"--list" {
            let pat = if a.len() > 1 {
                Some(String::from_utf8_lossy(a[1]).into_owned())
            } else {
                None
            };
            let text = std::fs::read_to_string(dpkg_status_db()).unwrap_or_default();
            let out = std::io::stdout();
            let mut lock = out.lock();
            let _ = writeln!(lock, "Desired=Unknown/Install/Remove/Purge/Hold");
            let mut pkg = String::new();
            let mut status = String::new();
            let mut ver = String::new();
            let mut desc = String::new();
            let flush = |pkg: &str, status: &str, ver: &str, desc: &str, lock: &mut dyn Write| {
                let ok = status.contains("installed");
                let _ = writeln!(
                    lock,
                    "{}  {pkg:<30} {ver:<16} {desc}",
                    if ok { "ii" } else { "un" }
                );
            };
            for line in text.lines().chain([""]) {
                if line.is_empty() {
                    if !pkg.is_empty() {
                        let hit = pat.as_ref().is_none_or(|p| pkg.contains(p));
                        if hit {
                            flush(&pkg, &status, &ver, &desc, &mut lock);
                        }
                    }
                    pkg.clear();
                    status.clear();
                    ver.clear();
                    desc.clear();
                } else if let Some((k, v)) = line.split_once(':') {
                    match k {
                        "Package" => pkg = v.trim().to_owned(),
                        "Status" => status = v.trim().to_owned(),
                        "Version" => ver = v.trim().to_owned(),
                        "Description" => desc = v.trim().to_owned(),
                        _ => {}
                    }
                }
            }
            return Ok(0);
        }
        if act == b"-s" || act == b"--status" {
            if a.len() < 2 {
                eprintln!("dpkg: -s needs a package name");
                return Ok(1);
            }
            let want = String::from_utf8_lossy(a[1]).into_owned();
            let text = match std::fs::read_to_string(dpkg_status_db()) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("dpkg: status database unavailable");
                    return Ok(1);
                }
            };
            let mut block = String::new();
            let mut found = false;
            for line in text.lines().chain([""]) {
                if line.is_empty() {
                    if block.lines().any(|l| l == format!("Package: {want}")) {
                        println!("{block}");
                        found = true;
                    }
                    block.clear();
                } else {
                    block.push_str(line);
                    block.push('\n');
                }
            }
            if !found {
                eprintln!("dpkg: package '{want}' not installed");
                return Ok(1);
            }
            return Ok(0);
        }
        if act == b"-i" || act == b"--install" || act == b"--unpack" {
            if a.len() < 2 {
                eprintln!("dpkg: --install needs a deb file");
                return Ok(1);
            }
            let mut rc = 0;
            for f in &args[1..] {
                let data = match std::fs::read(Path::new(f)) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("dpkg: {}: {e}", f.to_string_lossy());
                        rc = 1;
                        continue;
                    }
                };
                match deb_parts(&data) {
                    Ok((_, payload)) => {
                        let root = std::env::var_os("DPKG_ROOT")
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from("/"));
                        if extract_ustar(&payload, &root, false).is_err() {
                            eprintln!("dpkg: failed to unpack {}", f.to_string_lossy());
                            rc = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("dpkg: {}: {e}", f.to_string_lossy());
                        rc = 1;
                    }
                }
            }
            return Ok(rc);
        }
        eprintln!("dpkg: unsupported action {}", act.escape_ascii());
        Ok(1)
    }
}
