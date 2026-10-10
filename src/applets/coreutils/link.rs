use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

pub fn canonicalize_coreutils(path: &Path) -> Option<PathBuf> {
    if let Some(res) = sys_realpath(path) {
        return Some(res);
    }
    let err = io::Error::last_os_error();
    if err.raw_os_error() != Some(libc::ENOENT) {
        return None;
    }

    if let Ok(target) = fs::read_link(path) {
        let target_path = if target.is_relative() {
            if let Ok(cwd) = std::env::current_dir() {
                cwd.join(target)
            } else {
                target
            }
        } else {
            target
        };
        return canonicalize_coreutils(&target_path);
    }

    let bytes = path.as_os_str().as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut start = 0;
    while start + 1 < bytes.len() && bytes[start] == b'/' && bytes[start + 1] == b'/' {
        start += 1;
    }
    let mut end = bytes.len();
    while end > start + 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    let trimmed = &bytes[start..end];
    if trimmed.is_empty() {
        return None;
    }

    let last_slash = trimmed.iter().rposition(|&b| b == b'/');
    match last_slash {
        Some(0) => {
            let last_comp = &trimmed[1..];
            if last_comp.is_empty() {
                Some(PathBuf::from("/"))
            } else {
                let mut res = PathBuf::from("/");
                res.push(OsStr::from_bytes(last_comp));
                Some(res)
            }
        }
        Some(idx) => {
            let parent_bytes = &trimmed[..idx];
            let last_comp = &trimmed[idx + 1..];
            let parent_path = Path::new(OsStr::from_bytes(parent_bytes));
            if let Some(parent_res) = sys_realpath(parent_path) {
                let mut res = parent_res;
                res.push(OsStr::from_bytes(last_comp));
                Some(res)
            } else {
                None
            }
        }
        None => {
            if let Ok(cwd) = std::env::current_dir() {
                if let Some(parent_res) = sys_realpath(&cwd) {
                    let mut res = parent_res;
                    res.push(OsStr::from_bytes(trimmed));
                    Some(res)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

fn sys_realpath(path: &Path) -> Option<PathBuf> {
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    unsafe {
        let res = libc::realpath(c_path.as_ptr(), std::ptr::null_mut());
        if res.is_null() {
            None
        } else {
            let s = CStr::from_ptr(res).to_bytes().to_vec();
            libc::free(res as *mut libc::c_void);
            Some(PathBuf::from(OsString::from_vec(s)))
        }
    }
}

pub struct ReadlinkApplet;

impl Applet for ReadlinkApplet {
    fn name(&self) -> &'static str {
        "readlink"
    }

    fn description(&self) -> &'static str {
        "Display the value of a symlink"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_f = false;
        let mut opt_n = false;
        let mut opt_v = false;
        let mut file_opt: Option<&OsString> = None;

        let mut i = 0;
        let mut parsing_opts = true;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();

            if parsing_opts && bytes == b"--" {
                parsing_opts = false;
                i += 1;
                continue;
            }

            if parsing_opts && bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'f' => opt_f = true,
                        b'n' => opt_n = true,
                        b'v' => opt_v = true,
                        b's' | b'q' => {}
                        _ => {
                            eprintln!("readlink: invalid option -- '{}'", b as char);
                            eprintln!("Usage: readlink [-fnv] FILE");
                            return Ok(1);
                        }
                    }
                }
                i += 1;
            } else if file_opt.is_none() {
                file_opt = Some(arg);
                i += 1;
            } else {
                eprintln!("Usage: readlink [-fnv] FILE");
                return Ok(1);
            }
        }

        let file_arg = match file_opt {
            Some(f) => f,
            None => {
                eprintln!("Usage: readlink [-fnv] FILE");
                return Ok(1);
            }
        };

        let stdout = io::stdout();
        let mut out = stdout.lock();

        if opt_f {
            match canonicalize_coreutils(Path::new(file_arg)) {
                Some(p) => {
                    out.write_all(p.as_os_str().as_bytes())?;
                    if !opt_n {
                        out.write_all(b"\n")?;
                    }
                    let _ = out.flush();
                    Ok(0)
                }
                None => {
                    if opt_v {
                        let err = io::Error::last_os_error();
                        eprintln!(
                            "readlink: {}: cannot read link: {}",
                            file_arg.to_string_lossy(),
                            err
                        );
                    }
                    Ok(1)
                }
            }
        } else {
            match fs::read_link(Path::new(file_arg)) {
                Ok(target) => {
                    out.write_all(target.as_os_str().as_bytes())?;
                    if !opt_n {
                        out.write_all(b"\n")?;
                    }
                    let _ = out.flush();
                    Ok(0)
                }
                Err(e) => {
                    if opt_v {
                        let errmsg = if e.raw_os_error() == Some(libc::EINVAL) {
                            "not a symlink".to_string()
                        } else {
                            e.to_string()
                        };
                        eprintln!(
                            "readlink: {}: cannot read link: {}",
                            file_arg.to_string_lossy(),
                            errmsg
                        );
                    }
                    Ok(1)
                }
            }
        }
    }
}

pub struct RealpathApplet;

impl Applet for RealpathApplet {
    fn name(&self) -> &'static str {
        "realpath"
    }

    fn description(&self) -> &'static str {
        "Print absolute pathnames of FILEs"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: realpath FILE...");
            return Ok(1);
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut exit_code = 0;

        for arg in args {
            match canonicalize_coreutils(Path::new(arg)) {
                Some(p) => {
                    out.write_all(p.as_os_str().as_bytes())?;
                    out.write_all(b"\n")?;
                }
                None => {
                    exit_code = 1;
                    eprintln!(
                        "realpath: {}: No such file or directory",
                        arg.to_string_lossy()
                    );
                }
            }
        }

        let _ = out.flush();
        Ok(exit_code)
    }
}
