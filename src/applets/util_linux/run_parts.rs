use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

fn valid_runpart(name: &[u8]) -> bool {
    !name.is_empty()
        && name
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

pub struct RunPartsApplet;
impl Applet for RunPartsApplet {
    fn name(&self) -> &'static str {
        "run-parts"
    }
    fn description(&self) -> &'static str {
        "Run scripts in a directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut test = false;
        let mut verbose = false;
        let mut reverse = false;
        let mut umask_v: Option<u32> = None;
        let mut pass_args: Vec<OsString> = Vec::new();
        let mut dir: Option<&[u8]> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" || b == b"--test" {
                test = true;
            } else if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if b == b"--reverse" {
                reverse = true;
            } else if b == b"-a" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-parts: -a needs an argument");
                    return Ok(1);
                }
                pass_args.push(args[i].clone());
            } else if b == b"-u" || b == b"--umask" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-parts: -u needs a mask");
                    return Ok(1);
                }
                match u32::from_str_radix(std::str::from_utf8(args[i].as_bytes()).unwrap_or(""), 8)
                {
                    Ok(m) => umask_v = Some(m),
                    Err(_) => {
                        eprintln!("run-parts: invalid umask");
                        return Ok(1);
                    }
                }
            } else if b.first() == Some(&b'-') {
                eprintln!("run-parts: unknown option");
                return Ok(1);
            } else if dir.is_none() {
                dir = Some(b);
            } else {
                pass_args.push(args[i].clone());
            }
            i += 1;
        }
        let dir = match dir {
            Some(d) => d,
            None => {
                eprintln!("usage: run-parts [-t] [-v] [-a ARG] DIR");
                return Ok(1);
            }
        };
        if let Some(m) = umask_v {
            unsafe { libc::umask(m as libc::mode_t) };
        }
        let ds = match std::str::from_utf8(dir) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("run-parts: bad directory");
                return Ok(1);
            }
        };
        let mut entries: Vec<OsString> = Vec::new();
        let rd = match std::fs::read_dir(ds) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("run-parts: '{}': {}", ds, e);
                return Ok(1);
            }
        };
        for e in rd.flatten() {
            let n = e.file_name();
            if !valid_runpart(n.as_bytes()) {
                continue;
            }

            let p = e.path();
            let acc = {
                use std::ffi::CString;
                match CString::new(p.as_os_str().as_bytes()) {
                    Ok(c) => unsafe { libc::access(c.as_ptr(), libc::X_OK) == 0 },
                    Err(_) => false,
                }
            };
            if acc && p.is_file() {
                entries.push(n);
            }
        }
        entries.sort();
        if reverse {
            entries.reverse();
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for n in &entries {
            let mut full = Vec::with_capacity(ds.len() + n.len() + 2);
            full.extend_from_slice(ds.as_bytes());
            full.push(b'/');
            full.extend_from_slice(n.as_bytes());
            if test || verbose {
                out.write_all(&full)?;
                out.write_all(b"\n")?;
            }
            if test {
                continue;
            }

            let pid = unsafe { libc::fork() };
            if pid < 0 {
                eprintln!("run-parts: fork: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            if pid == 0 {
                use std::ffi::CString;
                let prog = CString::new(full.clone()).unwrap();
                let mut cs: Vec<CString> = Vec::with_capacity(pass_args.len() + 2);
                cs.push(prog.clone());
                for a in &pass_args {
                    cs.push(
                        CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()),
                    );
                }
                let mut ptrs: Vec<*const libc::c_char> = cs.iter().map(|c| c.as_ptr()).collect();
                ptrs.push(std::ptr::null());
                unsafe {
                    libc::execv(prog.as_ptr(), ptrs.as_ptr());
                    libc::_exit(127);
                }
            }
            let mut st = 0;
            unsafe {
                while libc::waitpid(pid, &mut st, 0) < 0
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                {
                }
                if !(libc::WIFEXITED(st) && libc::WEXITSTATUS(st) == 0) {
                    rc = 1;
                }
            }
        }
        out.flush()?;
        Ok(rc)
    }
}
