use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

applet!(
    MakedevsApplet,
    "makedevs",
    "Create device nodes from table",
    run_makedevs
);
fn run_makedevs(args: &[OsString]) -> Result<i32> {
    let (mut table, mut root) = (None::<OsString>, String::from("/"));
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" {
            i += 1;
            if i >= args.len() {
                eprintln!("makedevs: -d needs an argument");
                return Ok(1);
            }
            table = Some(args[i].clone());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("makedevs: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else if root == "/" {
            root = lossy(&args[i]);
        } else {
            eprintln!("makedevs: too many arguments");
            return Ok(1);
        }
        i += 1;
    }
    let data = match &table {
        Some(t) => match read_all(t.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("makedevs: can't open '{}': {}", lossy(t), e);
                return Ok(1);
            }
        },
        None => {
            eprintln!("makedevs: needs -d TABLE");
            return Ok(1);
        }
    };
    let mut rc = 0;
    for (ln, raw) in String::from_utf8_lossy(&data).lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 2 {
            eprintln!("makedevs: line {}: bad format", ln + 1);
            rc = 1;
            continue;
        }
        let path = format!(
            "{}{}{}",
            root.trim_end_matches('/'),
            if root.ends_with('/') { "" } else { "/" },
            f[0]
        );
        match f[1] {
            "d" => {
                let mode = f
                    .get(2)
                    .and_then(|s| u32::from_str_radix(s.trim_start_matches("0o"), 8).ok())
                    .unwrap_or(0o755);
                if std::fs::create_dir_all(&path).is_err()
                    || std::fs::set_permissions(
                        &path,
                        std::os::unix::fs::PermissionsExt::from_mode(mode),
                    )
                    .is_err()
                {
                    eprintln!("makedevs: line {}: mkdir failed", ln + 1);
                    rc = 1;
                }
            }
            "f" => {
                let mode = f
                    .get(2)
                    .and_then(|s| u32::from_str_radix(s, 8).ok())
                    .unwrap_or(0o644);
                match File::create(&path) {
                    Ok(_) => {
                        let _ = std::fs::set_permissions(
                            &path,
                            std::os::unix::fs::PermissionsExt::from_mode(mode),
                        );
                    }
                    Err(_) => {
                        eprintln!("makedevs: line {}: create failed", ln + 1);
                        rc = 1;
                    }
                }
            }
            "p" => {
                let c = match std::ffi::CString::new(path.as_bytes()) {
                    Ok(c) => c,
                    Err(_) => {
                        rc = 1;
                        continue;
                    }
                };
                if unsafe { libc::mkfifo(c.as_ptr(), 0o644) } != 0
                    && std::io::Error::last_os_error().kind() != std::io::ErrorKind::AlreadyExists
                {
                    eprintln!("makedevs: line {}: mkfifo failed", ln + 1);
                    rc = 1;
                }
            }
            "s" => {
                if f.len() < 3 {
                    eprintln!("makedevs: line {}: symlink needs target", ln + 1);
                    rc = 1;
                    continue;
                }
                if std::os::unix::fs::symlink(f[2], &path).is_err() {
                    eprintln!("makedevs: line {}: symlink failed", ln + 1);
                    rc = 1;
                }
            }
            "c" | "b" => {
                if f.len() < 8 {
                    eprintln!(
                        "makedevs: line {}: device needs maj min mode uid gid",
                        ln + 1
                    );
                    rc = 1;
                    continue;
                }
                let maj: u32 = f[2].parse().unwrap_or(0);
                let min: u32 = f[3].parse().unwrap_or(0);
                let mode = u32::from_str_radix(f[4], 8).unwrap_or(0o660);
                let kind = if f[1] == "c" {
                    libc::S_IFCHR
                } else {
                    libc::S_IFBLK
                };
                let c = match std::ffi::CString::new(path.as_bytes()) {
                    Ok(c) => c,
                    Err(_) => {
                        rc = 1;
                        continue;
                    }
                };
                if unsafe { libc::mknod(c.as_ptr(), kind | mode, libc::makedev(maj, min)) } != 0 {
                    eprintln!(
                        "makedevs: line {}: mknod failed: {}",
                        ln + 1,
                        std::io::Error::last_os_error()
                    );
                    rc = 1;
                    continue;
                }
                let uid: u32 = f[5].parse().unwrap_or(0);
                let gid: u32 = f[6].parse().unwrap_or(0);
                unsafe {
                    libc::chown(c.as_ptr(), uid, gid);
                }
            }
            t => {
                eprintln!("makedevs: line {}: unknown type '{}'", ln + 1, t);
                rc = 1;
            }
        }
    }
    Ok(rc)
}

