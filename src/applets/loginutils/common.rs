use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub fn is_help(a: &OsString) -> bool {
    a.as_bytes() == b"--help"
}

pub fn help_out(name: &str, usage: &str, desc: &str) -> Result<i32> {
    let stdout = io::stdout();
    let mut o = stdout.lock();
    o.write_all(b"Usage: ")?;
    o.write_all(name.as_bytes())?;
    o.write_all(b" ")?;
    o.write_all(usage.as_bytes())?;
    o.write_all(b"\n")?;
    o.write_all(desc.as_bytes())?;
    o.write_all(b"\n")?;
    Ok(0)
}

pub fn cstr(bytes: &[u8]) -> Option<CString> {
    CString::new(bytes).ok()
}

pub fn no_crypt(applet: &str) -> Result<i32> {
    eprintln!("{}: no crypt(3) in this build, refusing", applet);
    Ok(1)
}

pub fn acct_path(var: &str, def: &str) -> std::path::PathBuf {
    std::env::var_os(var)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(def))
}

pub fn passwd_path() -> std::path::PathBuf {
    acct_path("BB_PASSWD", "/etc/passwd")
}
pub fn shadow_path() -> std::path::PathBuf {
    acct_path("BB_SHADOW", "/etc/shadow")
}
pub fn group_path() -> std::path::PathBuf {
    acct_path("BB_GROUP", "/etc/group")
}
pub fn shells_path() -> std::path::PathBuf {
    acct_path("BB_SHELLS", "/etc/shells")
}

#[derive(Clone)]
pub struct PasswdEnt {
    pub name: Vec<u8>,
    pub pass: Vec<u8>,
    pub uid: u32,
    pub gid: u32,
    pub gecos: Vec<u8>,
    pub home: Vec<u8>,
    pub shell: Vec<u8>,
}

pub fn parse_passwd(data: &[u8]) -> Vec<PasswdEnt> {
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let f: Vec<&[u8]> = line.split(|&b| b == b':').collect();
        if f.len() != 7 {
            continue;
        }
        let (uid, gid) = match (
            std::str::from_utf8(f[2])
                .ok()
                .and_then(|s| s.parse::<u32>().ok()),
            std::str::from_utf8(f[3])
                .ok()
                .and_then(|s| s.parse::<u32>().ok()),
        ) {
            (Some(u), Some(g)) => (u, g),
            _ => continue,
        };
        out.push(PasswdEnt {
            name: f[0].to_vec(),
            pass: f[1].to_vec(),
            uid,
            gid,
            gecos: f[4].to_vec(),
            home: f[5].to_vec(),
            shell: f[6].to_vec(),
        });
    }
    out
}

pub fn render_passwd(ents: &[PasswdEnt]) -> Vec<u8> {
    let mut out = Vec::new();
    for e in ents {
        out.extend_from_slice(&e.name);
        out.push(b':');
        out.extend_from_slice(&e.pass);
        out.push(b':');
        out.extend_from_slice(e.uid.to_string().as_bytes());
        out.push(b':');
        out.extend_from_slice(e.gid.to_string().as_bytes());
        out.push(b':');
        out.extend_from_slice(&e.gecos);
        out.push(b':');
        out.extend_from_slice(&e.home);
        out.push(b':');
        out.extend_from_slice(&e.shell);
        out.push(b'\n');
    }
    out
}

pub fn lookup_user(name: &[u8]) -> Option<PasswdEnt> {
    let data = fs::read(passwd_path()).unwrap_or_default();
    parse_passwd(&data).into_iter().find(|e| e.name == name)
}

pub fn lookup_uid(uid: u32) -> Option<PasswdEnt> {
    let data = fs::read(passwd_path()).unwrap_or_default();
    parse_passwd(&data).into_iter().find(|e| e.uid == uid)
}

pub fn read_shadow_raw() -> Vec<Vec<Vec<u8>>> {
    let data = fs::read(shadow_path()).unwrap_or_default();
    data.split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| l.split(|&b| b == b':').map(|f| f.to_vec()).collect())
        .collect()
}

pub fn shadow_hash(name: &[u8]) -> Option<Vec<u8>> {
    for f in read_shadow_raw() {
        if f.len() >= 2 && f[0] == name {
            return Some(f[1].clone());
        }
    }

    lookup_user(name).and_then(|e| {
        if e.pass == b"x" || e.pass == b"*" {
            None
        } else {
            Some(e.pass)
        }
    })
}

pub fn atomic_replace(path: &std::path::Path, data: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("bbtmp");
    fs::write(&tmp, data)?;

    if path == shadow_path() {
        if let Some(c) = cstr(tmp.as_os_str().as_bytes()) {
            unsafe {
                libc::chmod(c.as_ptr(), 0o600);
            }
        }
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn is_locked_hash(h: &[u8]) -> bool {
    h.is_empty() || h.starts_with(b"!") || h.starts_with(b"*")
}

pub fn verify_password(hash: &[u8], clear: &[u8]) -> std::result::Result<bool, ()> {
    if hash.is_empty() {
        return Ok(clear.is_empty());
    }
    if is_locked_hash(hash) {
        return Ok(false);
    }

    Err(())
}

pub fn read_secret(prompt: &[u8], verify: bool) -> Option<Vec<u8>> {
    let _ = verify;
    let stdout = io::stdout();
    {
        let mut o = stdout.lock();
        let _ = o.write_all(prompt);
        let _ = o.flush();
    }

    let is_tty = unsafe { libc::isatty(0) == 1 };
    let mut saved: libc::termios = unsafe { std::mem::zeroed() };
    let have_term = if is_tty {
        unsafe {
            if libc::tcgetattr(0, &mut saved) == 0 {
                let mut t = saved;
                t.c_lflag &= !libc::ECHO;
                libc::tcsetattr(0, libc::TCSANOW, &t) == 0
            } else {
                false
            }
        }
    } else {
        false
    };
    let mut line = Vec::new();
    let mut b = [0u8; 1];
    let res = loop {
        match io::stdin().lock().read(&mut b) {
            Ok(0) => break None,
            Ok(_) => {
                if b[0] == b'\n' {
                    break Some(std::mem::take(&mut line));
                }
                if b[0] != b'\r' {
                    line.push(b[0]);
                }
            }
            Err(_) => break None,
        }
    };
    if have_term {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &saved);
        }
        let stdout = io::stdout();
        let mut o = stdout.lock();
        let _ = o.write_all(b"\n");
        let _ = o.flush();
    }
    res
}

pub fn user_groups(name: &[u8], pgid: u32) -> Vec<u32> {
    let mut gids = vec![pgid];
    let data = fs::read(group_path()).unwrap_or_default();
    for line in data.split(|&b| b == b'\n') {
        let f: Vec<&[u8]> = line.split(|&b| b == b':').collect();
        if f.len() < 4 {
            continue;
        }
        let gid: Option<u32> = std::str::from_utf8(f[2]).ok().and_then(|s| s.parse().ok());
        if let Some(g) = gid {
            if f[3].split(|&b| b == b',').any(|m| m == name) && !gids.contains(&g) {
                gids.push(g);
            }
        }
    }
    gids
}

pub fn become_and_exec(
    ent: &PasswdEnt,
    shell_override: Option<&[u8]>,
    login_dash: bool,
    cmd: Option<&[u8]>,
) -> Result<i32> {
    let shell = shell_override
        .map(|s| s.to_vec())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if ent.shell.is_empty() {
                b"/bin/sh".to_vec()
            } else {
                ent.shell.clone()
            }
        });
    if !shell.starts_with(b"/") {
        eprintln!("su: bad shell");
        return Ok(1);
    }
    let gids = user_groups(&ent.name, ent.gid);
    let gids_c: Vec<libc::gid_t> = gids.iter().map(|&g| g as libc::gid_t).collect();

    unsafe {
        if libc::setgroups(gids_c.len() as libc::size_t, gids_c.as_ptr()) != 0
            && libc::geteuid() == 0
        {
            eprintln!("su: setgroups: {}", io::Error::last_os_error());
            return Ok(1);
        }
        if libc::setgid(ent.gid) != 0 {
            eprintln!("su: setgid: {}", io::Error::last_os_error());
            return Ok(1);
        }
        if libc::setuid(ent.uid) != 0 {
            eprintln!("su: setuid: {}", io::Error::last_os_error());
            return Ok(1);
        }
    }
    let prog = match cstr(&shell) {
        Some(c) => c,
        None => {
            eprintln!("su: bad shell path");
            return Ok(1);
        }
    };

    unsafe {
        if let Some(c) = cmd {
            let a0 = cstr(b"sh").unwrap();
            let dashc = cstr(b"-c").unwrap();
            let cc = match cstr(c) {
                Some(x) => x,
                None => libc::_exit(1),
            };
            let argv = [a0.as_ptr(), dashc.as_ptr(), cc.as_ptr(), std::ptr::null()];
            libc::execv(prog.as_ptr(), argv.as_ptr());
        } else {
            let mut nm = Vec::with_capacity(shell.len() + 2);
            if login_dash {
                nm.push(b'-');
            }
            match shell.iter().rposition(|&b| b == b'/') {
                Some(p) => nm.extend_from_slice(&shell[p + 1..]),
                None => nm.extend_from_slice(&shell),
            }
            match cstr(&nm) {
                Some(a0) => {
                    let argv = [a0.as_ptr(), std::ptr::null()];
                    libc::execv(prog.as_ptr(), argv.as_ptr());
                }
                None => libc::_exit(1),
            }
        }
        eprintln!(
            "su: cannot exec {}: {}",
            String::from_utf8_lossy(&shell),
            io::Error::last_os_error()
        );
        libc::_exit(1);
    }
}



pub struct MkpasswdApplet;
impl Applet for MkpasswdApplet {
    fn name(&self) -> &'static str {
        "mkpasswd"
    }
    fn description(&self) -> &'static str {
        "Hash a password with crypt(3) (unavailable: fails loudly)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "mkpasswd",
                    "[-P FD] [-m TYPE] [-S SALT] [PASS] [SALT]",
                    "Print a crypted password (needs crypt)",
                );
            } else if b == b"-P" || b == b"-m" || b == b"-S" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mkpasswd: option requires an argument");
                    return Ok(1);
                }
            } else if b.starts_with(b"-") {
                eprintln!("mkpasswd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        no_crypt("mkpasswd")
    }
}

pub fn next_free_id(used: &[u32], system: bool) -> u32 {
    let (lo, hi) = if system { (100, 999) } else { (1000, 60000) };

    for id in lo..=hi {
        if !used.contains(&id) {
            return id;
        }
    }
    if system {
        for id in 1000..=60000 {
            if !used.contains(&id) {
                return id;
            }
        }
    }
    60001 + used.len() as u32
}



pub fn read_shells() -> Vec<Vec<u8>> {
    let data = fs::read(shells_path()).unwrap_or_default();
    data.split(|&b| b == b'\n')
        .filter(|l| {
            let t = l
                .iter()
                .position(|&b| b != b' ' && b != b'\t')
                .map(|i| &l[i..])
                .unwrap_or(b"");
            !t.is_empty() && !t.starts_with(b"#")
        })
        .map(|l| l.to_vec())
        .collect()
}
