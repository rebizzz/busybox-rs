use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

fn is_help(a: &OsString) -> bool {
    a.as_bytes() == b"--help"
}

fn help_out(name: &str, usage: &str, desc: &str) -> Result<i32> {
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

fn cstr(bytes: &[u8]) -> Option<CString> {
    CString::new(bytes).ok()
}

fn no_crypt(applet: &str) -> Result<i32> {
    eprintln!("{}: no crypt(3) in this build, refusing", applet);
    Ok(1)
}

fn acct_path(var: &str, def: &str) -> std::path::PathBuf {
    std::env::var_os(var)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(def))
}

fn passwd_path() -> std::path::PathBuf {
    acct_path("BB_PASSWD", "/etc/passwd")
}
fn shadow_path() -> std::path::PathBuf {
    acct_path("BB_SHADOW", "/etc/shadow")
}
fn group_path() -> std::path::PathBuf {
    acct_path("BB_GROUP", "/etc/group")
}
fn shells_path() -> std::path::PathBuf {
    acct_path("BB_SHELLS", "/etc/shells")
}

#[derive(Clone)]
struct PasswdEnt {
    name: Vec<u8>,
    pass: Vec<u8>,
    uid: u32,
    gid: u32,
    gecos: Vec<u8>,
    home: Vec<u8>,
    shell: Vec<u8>,
}

fn parse_passwd(data: &[u8]) -> Vec<PasswdEnt> {
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

fn render_passwd(ents: &[PasswdEnt]) -> Vec<u8> {
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

fn lookup_user(name: &[u8]) -> Option<PasswdEnt> {
    let data = fs::read(passwd_path()).unwrap_or_default();
    parse_passwd(&data).into_iter().find(|e| e.name == name)
}

fn lookup_uid(uid: u32) -> Option<PasswdEnt> {
    let data = fs::read(passwd_path()).unwrap_or_default();
    parse_passwd(&data).into_iter().find(|e| e.uid == uid)
}

fn read_shadow_raw() -> Vec<Vec<Vec<u8>>> {
    let data = fs::read(shadow_path()).unwrap_or_default();
    data.split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| l.split(|&b| b == b':').map(|f| f.to_vec()).collect())
        .collect()
}

fn shadow_hash(name: &[u8]) -> Option<Vec<u8>> {
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

fn atomic_replace(path: &std::path::Path, data: &[u8]) -> io::Result<()> {
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

fn is_locked_hash(h: &[u8]) -> bool {
    h.is_empty() || h.starts_with(b"!") || h.starts_with(b"*")
}

fn verify_password(hash: &[u8], clear: &[u8]) -> std::result::Result<bool, ()> {
    if hash.is_empty() {
        return Ok(clear.is_empty());
    }
    if is_locked_hash(hash) {
        return Ok(false);
    }

    Err(())
}

fn read_secret(prompt: &[u8], verify: bool) -> Option<Vec<u8>> {
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

fn user_groups(name: &[u8], pgid: u32) -> Vec<u32> {
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

fn become_and_exec(
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

pub struct LoginApplet;
impl Applet for LoginApplet {
    fn name(&self) -> &'static str {
        "login"
    }
    fn description(&self) -> &'static str {
        "Log in as a user (subset: passwd/shadow auth, no crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut preserve = false;
        let mut force_user: Option<Vec<u8>> = None;
        let mut host: Option<Vec<u8>> = None;
        let mut user: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "login",
                    "[-p] [-h HOST] [-f USER] [USER]",
                    "Log in as a user",
                );
            } else if b == b"-p" {
                preserve = true;
            } else if b == b"-h" {
                i += 1;
                if i >= args.len() {
                    eprintln!("login: option requires an argument -- 'h'");
                    return Ok(1);
                }
                host = Some(args[i].as_bytes().to_vec());
            } else if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("login: option requires an argument -- 'f'");
                    return Ok(1);
                }
                force_user = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-") {
                eprintln!("login: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() && force_user.is_none() {
                user = Some(b.to_vec());
            } else {
                eprintln!("login: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let _ = preserve;
        let _ = host;

        let euid = unsafe { libc::geteuid() };
        let name: Vec<u8> = if let Some(f) = force_user {
            if euid != 0 {
                eprintln!("login: -f only for root");
                return Ok(1);
            }
            f
        } else if let Some(u) = user {
            u
        } else {
            let stdout = io::stdout();
            {
                let mut o = stdout.lock();
                o.write_all(b"login: ")?;
                o.flush()?;
            }
            let mut line = Vec::new();
            let mut b = [0u8; 1];
            loop {
                match io::stdin().lock().read(&mut b) {
                    Ok(0) => break,
                    Ok(_) => {
                        if b[0] == b'\n' {
                            break;
                        }
                        line.push(b[0]);
                    }
                    Err(_) => break,
                }
            }

            while line.last() == Some(&b' ')
                || line.last() == Some(&b'\t')
                || line.last() == Some(&b'\r')
            {
                line.pop();
            }
            line
        };
        if name.is_empty() || name.contains(&b':') || name.contains(&b' ') {
            eprintln!("login: invalid login name");
            return Ok(1);
        }
        let ent = match lookup_user(&name) {
            Some(e) => e,
            None => {
                let _ = read_secret(b"Password: ", false);
                eprintln!("login: unknown user '{}'", String::from_utf8_lossy(&name));
                return Ok(1);
            }
        };

        if ent.uid != 0 {
            let nologin = acct_path("BB_NOLOGIN", "/etc/nologin");
            if nologin.exists() {
                match fs::read(&nologin) {
                    Ok(d) => {
                        let stdout = io::stdout();
                        let mut o = stdout.lock();
                        o.write_all(&d)?;
                        if !d.ends_with(b"\n") {
                            o.write_all(b"\n")?;
                        }
                    }
                    Err(_) => {
                        eprintln!("login: system is not available");
                    }
                }
                return Ok(1);
            }
        }
        let hash = shadow_hash(&name).unwrap_or_default();
        let pw = match read_secret(b"Password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("login: input error");
                return Ok(1);
            }
        };
        match verify_password(&hash, &pw) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("login: incorrect password");
                return Ok(1);
            }
            Err(()) => return no_crypt("login"),
        }

        if !ent.home.is_empty() {
            if let Some(h) = cstr(&ent.home) {
                if unsafe { libc::chdir(h.as_ptr()) } != 0 {
                    eprintln!(
                        "login: cannot chdir to '{}'",
                        String::from_utf8_lossy(&ent.home)
                    );
                }
            }
        }
        become_and_exec(&ent, None, true, None)
    }
}

pub struct SuApplet;
impl Applet for SuApplet {
    fn name(&self) -> &'static str {
        "su"
    }
    fn description(&self) -> &'static str {
        "Switch user (subset: - -c -s; password needs crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut login_shell = false;
        let mut cmd: Option<Vec<u8>> = None;
        let mut shell: Option<Vec<u8>> = None;
        let mut user: Option<Vec<u8>> = None;
        let mut extra: Vec<Vec<u8>> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "su",
                    "[-] [-c CMD] [-s SHELL] [USER [ARGS...]]",
                    "Switch user identity",
                );
            } else if b == b"-" || b == b"-l" || b == b"--login" {
                login_shell = true;
            } else if b == b"-c" || b == b"--command" {
                i += 1;
                if i >= args.len() {
                    eprintln!("su: option requires an argument -- 'c'");
                    return Ok(1);
                }
                cmd = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-c") && b.len() > 2 {
                cmd = Some(b[2..].to_vec());
            } else if b == b"-s" || b == b"--shell" {
                i += 1;
                if i >= args.len() {
                    eprintln!("su: option requires an argument -- 's'");
                    return Ok(1);
                }
                shell = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-s") && b.len() > 2 {
                shell = Some(b[2..].to_vec());
            } else if b == b"--preserve-environment" {
            } else if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") {
                let mut ok = true;
                for &c in &b[1..] {
                    match c {
                        b'l' => login_shell = true,
                        b'p' | b'm' => {}
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    eprintln!("su: invalid option '{}'", String::from_utf8_lossy(b));
                    return Ok(1);
                }
            } else if b.starts_with(b"-") {
                eprintln!("su: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() {
                user = Some(b.to_vec());
            } else {
                extra.push(b.to_vec());
            }
            i += 1;
        }
        let _ = extra;
        let target = user.unwrap_or_else(|| b"root".to_vec());
        let ent = match lookup_user(&target) {
            Some(e) => e,
            None => {
                eprintln!("su: unknown user '{}'", String::from_utf8_lossy(&target));
                return Ok(1);
            }
        };

        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            let hash = shadow_hash(&target).unwrap_or_default();
            let pw = match read_secret(b"Password: ", false) {
                Some(p) => p,
                None => {
                    eprintln!("su: input error");
                    return Ok(1);
                }
            };
            match verify_password(&hash, &pw) {
                Ok(true) => {}
                Ok(false) => {
                    eprintln!("su: incorrect password");
                    return Ok(1);
                }
                Err(()) => return no_crypt("su"),
            }
        }
        if login_shell && !ent.home.is_empty() {
            if let Some(h) = cstr(&ent.home) {
                if unsafe { libc::chdir(h.as_ptr()) } != 0 {
                    eprintln!(
                        "su: cannot chdir to '{}'",
                        String::from_utf8_lossy(&ent.home)
                    );
                }
            }
        }
        become_and_exec(&ent, shell.as_deref(), login_shell, cmd.as_deref())
    }
}

pub struct SuloginApplet;
impl Applet for SuloginApplet {
    fn name(&self) -> &'static str {
        "sulogin"
    }
    fn description(&self) -> &'static str {
        "Single-user login (subset: root password, no crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut timeout: u64 = 0;
        let mut tty: Option<&OsString> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out("sulogin", "[-t SEC] [TTY]", "Single-user root login");
            } else if b == b"-t" {
                i += 1;
                if i >= args.len() {
                    eprintln!("sulogin: option requires an argument -- 't'");
                    return Ok(1);
                }
                timeout = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if b == b"-p" {
            } else if b.starts_with(b"-") {
                eprintln!("sulogin: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if tty.is_none() {
                tty = Some(&args[i]);
            } else {
                eprintln!("sulogin: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let _ = timeout;
        let _ = tty;
        let root = lookup_user(b"root").unwrap_or(PasswdEnt {
            name: b"root".to_vec(),
            pass: b"x".to_vec(),
            uid: 0,
            gid: 0,
            gecos: Vec::new(),
            home: b"/root".to_vec(),
            shell: b"/bin/sh".to_vec(),
        });
        let hash = shadow_hash(b"root").unwrap_or_default();
        let pw = match read_secret(
            b"Give root password for maintenance (or press Ctrl-D to continue): ",
            false,
        ) {
            Some(p) => p,
            None => {
                if hash.is_empty() {
                    return become_and_exec(&root, Some(b"/bin/sh"), false, None);
                }
                eprintln!("sulogin: no password given");
                return Ok(1);
            }
        };
        match verify_password(&hash, &pw) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("sulogin: incorrect password");
                return Ok(1);
            }
            Err(()) => return no_crypt("sulogin"),
        }
        become_and_exec(&root, Some(b"/bin/sh"), false, None)
    }
}

pub struct PasswdApplet;
impl Applet for PasswdApplet {
    fn name(&self) -> &'static str {
        "passwd"
    }
    fn description(&self) -> &'static str {
        "Change passwords (subset: -l -u -d real; change needs crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lock = false;
        let mut unlock = false;
        let mut delete = false;
        let mut user: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "passwd",
                    "[-a ALG] [-l|-u|-d] [USER]",
                    "Lock/unlock/change passwords",
                );
            } else if b == b"-a" {
                i += 1;
                if i >= args.len() {
                    eprintln!("passwd: option requires an argument -- 'a'");
                    return Ok(1);
                }
            } else if b == b"-l" || b == b"--lock" {
                lock = true;
            } else if b == b"-u" || b == b"--unlock" {
                unlock = true;
            } else if b == b"-d" || b == b"--delete" {
                delete = true;
            } else if b.starts_with(b"-") {
                eprintln!("passwd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() {
                user = Some(b.to_vec());
            } else {
                eprintln!("passwd: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let nmodes = [lock, unlock, delete].iter().filter(|&&x| x).count();
        if nmodes > 1 {
            eprintln!("passwd: only one of -l, -u, -d");
            return Ok(1);
        }

        let euid = unsafe { libc::geteuid() };
        let me = lookup_uid(unsafe { libc::getuid() })
            .map(|e| e.name)
            .unwrap_or_else(|| b"root".to_vec());
        let target = user.unwrap_or_else(|| me.clone());
        if target != me && euid != 0 {
            eprintln!("passwd: must be privileged to change others");
            return Ok(1);
        }
        if lookup_user(&target).is_none() {
            eprintln!(
                "passwd: unknown user '{}'",
                String::from_utf8_lossy(&target)
            );
            return Ok(1);
        }
        if lock || unlock || delete {
            let mut lines = read_shadow_raw();
            let has_shadow = !lines.is_empty();
            let mut found = false;
            for f in lines.iter_mut() {
                if f.len() >= 2 && f[0] == target {
                    found = true;
                    if lock {
                        if !f[1].starts_with(b"!") {
                            let mut nh = Vec::with_capacity(f[1].len() + 1);
                            nh.push(b'!');
                            nh.extend_from_slice(&f[1]);
                            f[1] = nh;
                        }
                    } else if unlock {
                        if f[1].starts_with(b"!") {
                            f[1] = f[1][1..].to_vec();
                        }
                    } else {
                        f[1].clear();
                    }
                }
            }
            if !found {
                if has_shadow {
                    eprintln!(
                        "passwd: no shadow entry for '{}'",
                        String::from_utf8_lossy(&target)
                    );
                    return Ok(1);
                }

                let data = fs::read(passwd_path()).unwrap_or_default();
                let mut ents = parse_passwd(&data);
                let mut hit = false;
                for e in ents.iter_mut() {
                    if e.name == target {
                        hit = true;
                        if lock {
                            if !e.pass.starts_with(b"!") {
                                let mut nh = vec![b'!'];
                                nh.extend_from_slice(&e.pass);
                                e.pass = nh;
                            }
                        } else if unlock {
                            if e.pass.starts_with(b"!") {
                                e.pass = e.pass[1..].to_vec();
                            }
                        } else {
                            e.pass.clear();
                        }
                    }
                }
                if !hit {
                    eprintln!(
                        "passwd: unknown user '{}'",
                        String::from_utf8_lossy(&target)
                    );
                    return Ok(1);
                }
                if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&ents)) {
                    eprintln!("passwd: cannot update passwd: {}", e);
                    return Ok(1);
                }
                return Ok(0);
            }
            let mut out = Vec::new();
            for f in &lines {
                for (k, fld) in f.iter().enumerate() {
                    if k > 0 {
                        out.push(b':');
                    }
                    out.extend_from_slice(fld);
                }
                out.push(b'\n');
            }
            if let Err(e) = atomic_replace(&shadow_path(), &out) {
                eprintln!("passwd: cannot update shadow: {}", e);
                return Ok(1);
            }
            return Ok(0);
        }

        let p1 = match read_secret(b"New password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("passwd: input error");
                return Ok(1);
            }
        };
        let p2 = match read_secret(b"Retype password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("passwd: input error");
                return Ok(1);
            }
        };
        if p1 != p2 {
            eprintln!("passwd: passwords do not match");
            return Ok(1);
        }
        no_crypt("passwd")
    }
}

pub struct ChpasswdApplet;
impl Applet for ChpasswdApplet {
    fn name(&self) -> &'static str {
        "chpasswd"
    }
    fn description(&self) -> &'static str {
        "Bulk password update (subset: -e passthrough real)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut encrypted = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "chpasswd",
                    "[-e] [-m] [-c ALG] [-R DIR]",
                    "Update passwords from USER:PASS lines on stdin",
                );
            } else if b == b"-e" || b == b"--encrypted" {
                encrypted = true;
            } else if b == b"-m" {
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("chpasswd: option requires an argument -- 'c'");
                    return Ok(1);
                }
            } else if b == b"-R" {
                i += 1;
                if i >= args.len() {
                    eprintln!("chpasswd: option requires an argument -- 'R'");
                    return Ok(1);
                }
                match cstr(args[i].as_bytes()) {
                    Some(d) => {
                        if unsafe { libc::chroot(d.as_ptr()) } != 0 {
                            eprintln!("chpasswd: chroot: {}", io::Error::last_os_error());
                            return Ok(1);
                        }
                    }
                    None => {
                        eprintln!("chpasswd: bad directory");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("chpasswd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        let mut input = Vec::new();
        io::stdin().lock().read_to_end(&mut input)?;
        let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for line in input.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                continue;
            }
            let mut sp = line.splitn(2, |&b| b == b':');
            let (u, p) = (sp.next().unwrap_or(b""), sp.next());
            let p = match p {
                Some(p) => p,
                None => {
                    eprintln!("chpasswd: bad line '{}'", String::from_utf8_lossy(line));
                    return Ok(1);
                }
            };
            if u.is_empty() {
                eprintln!("chpasswd: bad line '{}'", String::from_utf8_lossy(line));
                return Ok(1);
            }
            if p.is_empty() && !encrypted {
                eprintln!(
                    "chpasswd: empty password for '{}'",
                    String::from_utf8_lossy(u)
                );
                return Ok(1);
            }
            if lookup_user(u).is_none() {
                eprintln!("chpasswd: unknown user '{}'", String::from_utf8_lossy(u));
                return Ok(1);
            }
            pairs.push((u.to_vec(), p.to_vec()));
        }
        if !encrypted {
            return no_crypt("chpasswd");
        }
        let mut lines = read_shadow_raw();
        for (u, h) in &pairs {
            let mut hit = false;
            for f in lines.iter_mut() {
                if f.len() >= 2 && f[0] == *u {
                    f[1] = h.clone();
                    hit = true;
                }
            }
            if !hit {
                lines.push(vec![
                    u.clone(),
                    h.clone(),
                    b"18000".to_vec(),
                    b"0".to_vec(),
                    b"99999".to_vec(),
                    b"7".to_vec(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ]);
            }
        }
        let mut out = Vec::new();
        for f in &lines {
            for (k, fld) in f.iter().enumerate() {
                if k > 0 {
                    out.push(b':');
                }
                out.extend_from_slice(fld);
            }
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shadow_path(), &out) {
            eprintln!("chpasswd: cannot update shadow: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct CryptpwApplet;
impl Applet for CryptpwApplet {
    fn name(&self) -> &'static str {
        "cryptpw"
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
                    "cryptpw",
                    "[-P FD] [-m TYPE] [-S SALT] [PASS] [SALT]",
                    "Print a crypted password (needs crypt)",
                );
            } else if b == b"-P" || b == b"-m" || b == b"-S" {
                i += 1;
                if i >= args.len() {
                    eprintln!("cryptpw: option requires an argument");
                    return Ok(1);
                }
            } else if b.starts_with(b"-") {
                eprintln!("cryptpw: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        no_crypt("cryptpw")
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

fn next_free_id(used: &[u32], system: bool) -> u32 {
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

pub struct AdduserApplet;
impl Applet for AdduserApplet {
    fn name(&self) -> &'static str {
        "adduser"
    }
    fn description(&self) -> &'static str {
        "Create a user (subset: passwd/shadow/home)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut home: Option<Vec<u8>> = None;
        let mut no_home = false;
        let mut gecos = b"Linux User".to_vec();
        let mut shell = b"/bin/sh".to_vec();
        let mut extra_group: Option<Vec<u8>> = None;
        let mut system = false;
        let mut uid_opt: Option<u32> = None;
        let mut name: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "adduser",
                    "[-H] [-S] [-D] [-h DIR] [-g GECOS] [-s SHELL] [-G GRP] [-u UID] USER",
                    "Create a new user",
                );
            } else if b == b"-H" {
                no_home = true;
            } else if b == b"-S" {
                system = true;
            } else if b == b"-D" {
            } else if b == b"-h" || b == b"-g" || b == b"-s" || b == b"-G" || b == b"-u" {
                i += 1;
                if i >= args.len() {
                    eprintln!("adduser: option requires an argument");
                    return Ok(1);
                }
                let v = args[i].as_bytes().to_vec();
                match b {
                    b"-h" => home = Some(v),
                    b"-g" => gecos = v,
                    b"-s" => shell = v,
                    b"-G" => extra_group = Some(v),
                    _ => match std::str::from_utf8(&v)
                        .ok()
                        .and_then(|s| s.parse::<u32>().ok())
                    {
                        Some(n) => uid_opt = Some(n),
                        None => {
                            eprintln!("adduser: invalid UID");
                            return Ok(1);
                        }
                    },
                }
            } else if b.starts_with(b"-") {
                eprintln!("adduser: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if name.is_none() {
                name = Some(b.to_vec());
            } else {
                eprintln!("adduser: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let name = match name {
            Some(n) => n,
            None => {
                eprintln!("Usage: adduser [-H] [-S] [-h DIR] [-g GECOS] [-s SHELL] [-G GRP] [-u UID] USER");
                return Ok(1);
            }
        };
        if name.contains(&b':') || name.contains(&b'/') || name.is_empty() || name.len() > 32 {
            eprintln!("adduser: invalid user name");
            return Ok(1);
        }
        let data = fs::read(passwd_path()).unwrap_or_default();
        let mut ents = parse_passwd(&data);
        if ents.iter().any(|e| e.name == name) {
            eprintln!("adduser: user '{}' exists", String::from_utf8_lossy(&name));
            return Ok(1);
        }
        let used: Vec<u32> = ents.iter().map(|e| e.uid).collect();
        let uid = match uid_opt {
            Some(u) => {
                if used.contains(&u) {
                    eprintln!("adduser: UID {} in use", u);
                    return Ok(1);
                }
                u
            }
            None => next_free_id(&used, system),
        };

        let gdata = fs::read(group_path()).unwrap_or_default();
        let mut glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        let mygroup = String::from_utf8_lossy(&name).into_owned();
        let mut gid = uid;
        let mut have_group = false;
        for g in &glines {
            let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
            if f.len() >= 3 && f[0] == name.as_slice() {
                if let Ok(n) = std::str::from_utf8(f[2]).unwrap_or("").parse::<u32>() {
                    gid = n;
                    have_group = true;
                }
            }
        }
        if !have_group {
            let gused: Vec<u32> = glines
                .iter()
                .filter_map(|g| {
                    let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
                    if f.len() >= 3 {
                        std::str::from_utf8(f[2]).ok()?.parse().ok()
                    } else {
                        None
                    }
                })
                .collect();
            if gused.contains(&gid) {
                gid = next_free_id(&gused, system);
            }
            let mut nl = Vec::new();
            nl.extend_from_slice(&name);
            nl.push(b':');
            nl.extend_from_slice(b"x:");
            nl.extend_from_slice(gid.to_string().as_bytes());
            nl.extend_from_slice(b":");
            glines.push(nl);
        }
        if let Some(ref g) = extra_group {
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == g.as_slice() {
                    hit = true;
                    let members = f[3];
                    let already = members.split(|&b| b == b',').any(|m| m == name.as_slice());
                    if !already {
                        let mut ng = gl.clone();
                        if !members.is_empty() {
                            ng.push(b',');
                        }
                        ng.extend_from_slice(&name);
                        *gl = ng;
                    }
                }
            }
            if !hit {
                eprintln!("adduser: unknown group '{}'", String::from_utf8_lossy(g));
                return Ok(1);
            }
            let _ = mygroup;
        }
        let hdir = home.unwrap_or_else(|| {
            if no_home {
                Vec::new()
            } else {
                let mut h = b"/home/".to_vec();
                h.extend_from_slice(&name);
                h
            }
        });
        ents.push(PasswdEnt {
            name: name.clone(),
            pass: b"x".to_vec(),
            uid,
            gid,
            gecos,
            home: hdir.clone(),
            shell,
        });
        if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&ents)) {
            eprintln!("adduser: cannot update passwd: {}", e);
            return Ok(1);
        }

        let mut slines = read_shadow_raw();
        if !slines.iter().any(|f| !f.is_empty() && f[0] == name) {
            slines.push(vec![
                name.clone(),
                b"!".to_vec(),
                b"18000".to_vec(),
                b"0".to_vec(),
                b"99999".to_vec(),
                b"7".to_vec(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ]);
            let mut out = Vec::new();
            for f in &slines {
                for (k, fld) in f.iter().enumerate() {
                    if k > 0 {
                        out.push(b':');
                    }
                    out.extend_from_slice(fld);
                }
                out.push(b'\n');
            }
            if let Err(e) = atomic_replace(&shadow_path(), &out) {
                eprintln!("adduser: cannot update shadow: {}", e);
                return Ok(1);
            }
        }
        let mut gout = Vec::new();
        for g in &glines {
            gout.extend_from_slice(g);
            gout.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &gout) {
            eprintln!("adduser: cannot update group: {}", e);
            return Ok(1);
        }
        if !hdir.is_empty() && !no_home {
            let hp = std::path::Path::new(std::ffi::OsStr::from_bytes(&hdir));
            if let Err(e) = fs::create_dir_all(hp) {
                eprintln!("adduser: cannot create home: {}", e);
            } else {
                if let Some(c) = cstr(&hdir) {
                    unsafe {
                        libc::chown(c.as_ptr(), uid, gid);
                    }
                }
            }
        }
        Ok(0)
    }
}

pub struct DeluserApplet;
impl Applet for DeluserApplet {
    fn name(&self) -> &'static str {
        "deluser"
    }
    fn description(&self) -> &'static str {
        "Delete a user (subset: --remove-home, group member)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut remove_home = false;
        let mut pos: Vec<Vec<u8>> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if is_help(a) {
                return help_out(
                    "deluser",
                    "[--remove-home] USER [GROUP]",
                    "Delete a user or group membership",
                );
            } else if b == b"--remove-home" || b == b"-r" {
                remove_home = true;
            } else if b.starts_with(b"-") {
                eprintln!("deluser: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
        }
        if pos.is_empty() || pos.len() > 2 {
            eprintln!("Usage: deluser [--remove-home] USER [GROUP]");
            return Ok(1);
        }
        if pos.len() == 2 {
            let (user, grp) = (&pos[0], &pos[1]);
            let gdata = fs::read(group_path()).unwrap_or_default();
            let mut glines: Vec<Vec<u8>> = gdata
                .split(|&b| b == b'\n')
                .filter(|l| !l.is_empty())
                .map(|l| l.to_vec())
                .collect();
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == grp.as_slice() {
                    hit = true;
                    let members: Vec<&[u8]> = f[3]
                        .split(|&b| b == b',')
                        .filter(|m| *m != user.as_slice() && !m.is_empty())
                        .collect();
                    let mut ng = f[0].to_vec();
                    ng.push(b':');
                    ng.extend_from_slice(f[1]);
                    ng.push(b':');
                    ng.extend_from_slice(f[2]);
                    ng.push(b':');
                    ng.extend_from_slice(&members.join(&b','));
                    *gl = ng;
                }
            }
            if !hit {
                eprintln!("deluser: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
            let mut out = Vec::new();
            for g in &glines {
                out.extend_from_slice(g);
                out.push(b'\n');
            }
            if let Err(e) = atomic_replace(&group_path(), &out) {
                eprintln!("deluser: cannot update group: {}", e);
                return Ok(1);
            }
            return Ok(0);
        }
        let user = &pos[0];
        let data = fs::read(passwd_path()).unwrap_or_default();
        let ents = parse_passwd(&data);
        let gone: Vec<&PasswdEnt> = ents.iter().filter(|e| &e.name == user).collect();
        if gone.is_empty() {
            eprintln!("deluser: unknown user '{}'", String::from_utf8_lossy(user));
            return Ok(1);
        }
        let home = gone[0].home.clone();
        let keep: Vec<PasswdEnt> = ents.into_iter().filter(|e| &e.name != user).collect();
        if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&keep)) {
            eprintln!("deluser: cannot update passwd: {}", e);
            return Ok(1);
        }
        let slines: Vec<Vec<Vec<u8>>> = read_shadow_raw()
            .into_iter()
            .filter(|f| f.is_empty() || f[0] != *user)
            .collect();
        let mut out = Vec::new();
        for f in &slines {
            for (k, fld) in f.iter().enumerate() {
                if k > 0 {
                    out.push(b':');
                }
                out.extend_from_slice(fld);
            }
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shadow_path(), &out) {
            eprintln!("deluser: cannot update shadow: {}", e);
            return Ok(1);
        }
        if remove_home && !home.is_empty() {
            let hp = std::path::Path::new(std::ffi::OsStr::from_bytes(&home));
            if let Err(e) = fs::remove_dir_all(hp) {
                eprintln!("deluser: cannot remove home: {}", e);
                return Ok(1);
            }
        }
        Ok(0)
    }
}

pub struct AddgroupApplet;
impl Applet for AddgroupApplet {
    fn name(&self) -> &'static str {
        "addgroup"
    }
    fn description(&self) -> &'static str {
        "Create a group or add a user to one (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut gid_opt: Option<u32> = None;
        let mut system = false;
        let mut pos: Vec<Vec<u8>> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "addgroup",
                    "[-g GID] [-S] GROUP | USER GROUP",
                    "Create a group or add a member",
                );
            } else if b == b"-S" {
                system = true;
            } else if b == b"-g" {
                i += 1;
                if i >= args.len() {
                    eprintln!("addgroup: option requires an argument -- 'g'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(n) => gid_opt = Some(n),
                    None => {
                        eprintln!("addgroup: invalid GID");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"-") {
                eprintln!("addgroup: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
            i += 1;
        }
        let gdata = fs::read(group_path()).unwrap_or_default();
        let mut glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        if pos.len() == 2 {
            let (user, grp) = (&pos[0], &pos[1]);
            if lookup_user(user).is_none() {
                eprintln!("addgroup: unknown user '{}'", String::from_utf8_lossy(user));
                return Ok(1);
            }
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == grp.as_slice() {
                    hit = true;
                    if !f[3].split(|&b| b == b',').any(|m| m == user.as_slice()) {
                        let mut ng = gl.clone();
                        if !f[3].is_empty() {
                            ng.push(b',');
                        }
                        ng.extend_from_slice(user);
                        *gl = ng;
                    }
                }
            }
            if !hit {
                eprintln!("addgroup: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
        } else if pos.len() == 1 {
            let grp = &pos[0];
            if grp.contains(&b':') || grp.contains(&b'/') || grp.is_empty() {
                eprintln!("addgroup: invalid group name");
                return Ok(1);
            }
            if glines
                .iter()
                .any(|g| g.split(|&b| b == b':').next() == Some(grp.as_slice()))
            {
                eprintln!("addgroup: group '{}' exists", String::from_utf8_lossy(grp));
                return Ok(1);
            }
            let used: Vec<u32> = glines
                .iter()
                .filter_map(|g| {
                    let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
                    if f.len() >= 3 {
                        std::str::from_utf8(f[2]).ok()?.parse().ok()
                    } else {
                        None
                    }
                })
                .collect();
            let gid = match gid_opt {
                Some(g) => {
                    if used.contains(&g) {
                        eprintln!("addgroup: GID {} in use", g);
                        return Ok(1);
                    }
                    g
                }
                None => next_free_id(&used, system),
            };
            let mut nl = grp.clone();
            nl.extend_from_slice(b":x:");
            nl.extend_from_slice(gid.to_string().as_bytes());
            nl.extend_from_slice(b":");
            glines.push(nl);
        } else {
            eprintln!("Usage: addgroup [-g GID] [-S] GROUP | USER GROUP");
            return Ok(1);
        }
        let mut out = Vec::new();
        for g in &glines {
            out.extend_from_slice(g);
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &out) {
            eprintln!("addgroup: cannot update group: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct DelgroupApplet;
impl Applet for DelgroupApplet {
    fn name(&self) -> &'static str {
        "delgroup"
    }
    fn description(&self) -> &'static str {
        "Delete a group or remove a member (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pos: Vec<Vec<u8>> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if is_help(a) {
                return help_out(
                    "delgroup",
                    "GROUP | USER GROUP",
                    "Delete a group or a member",
                );
            } else if b.starts_with(b"-") {
                eprintln!("delgroup: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
        }
        if pos.is_empty() || pos.len() > 2 {
            eprintln!("Usage: delgroup GROUP | USER GROUP");
            return Ok(1);
        }
        let gdata = fs::read(group_path()).unwrap_or_default();
        let glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        let mut out: Vec<Vec<u8>> = Vec::new();
        let mut hit = false;
        if pos.len() == 1 {
            for g in &glines {
                if g.split(|&b| b == b':').next() == Some(pos[0].as_slice()) {
                    hit = true;
                } else {
                    out.push(g.clone());
                }
            }
            if !hit {
                eprintln!(
                    "delgroup: unknown group '{}'",
                    String::from_utf8_lossy(&pos[0])
                );
                return Ok(1);
            }
        } else {
            let (user, grp) = (&pos[0], &pos[1]);
            for g in &glines {
                let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == grp.as_slice() {
                    hit = true;
                    let members: Vec<&[u8]> = f[3]
                        .split(|&b| b == b',')
                        .filter(|m| *m != user.as_slice() && !m.is_empty())
                        .collect();
                    let mut ng = f[0].to_vec();
                    ng.push(b':');
                    ng.extend_from_slice(f[1]);
                    ng.push(b':');
                    ng.extend_from_slice(f[2]);
                    ng.push(b':');
                    ng.extend_from_slice(&members.join(&b','));
                    out.push(ng);
                } else {
                    out.push(g.clone());
                }
            }
            if !hit {
                eprintln!("delgroup: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
        }
        let mut data = Vec::new();
        for g in &out {
            data.extend_from_slice(g);
            data.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &data) {
            eprintln!("delgroup: cannot update group: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}

fn read_shells() -> Vec<Vec<u8>> {
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

pub struct AddShellApplet;
impl Applet for AddShellApplet {
    fn name(&self) -> &'static str {
        "add-shell"
    }
    fn description(&self) -> &'static str {
        "Add a shell to /etc/shells"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("add-shell", "SHELL...", "Add SHELLs to /etc/shells");
            }
        }
        if args.is_empty() {
            eprintln!("Usage: add-shell SHELL...");
            return Ok(1);
        }
        let mut shells = read_shells();
        for a in args {
            let sh = a.as_bytes();
            if !sh.starts_with(b"/") {
                eprintln!("add-shell: shell must be an absolute path");
                return Ok(1);
            }
            let sp = std::path::Path::new(std::ffi::OsStr::from_bytes(sh));
            match fs::metadata(sp) {
                Ok(m) if m.is_file() => {}
                _ => {
                    eprintln!("add-shell: '{}' is not a file", String::from_utf8_lossy(sh));
                    return Ok(1);
                }
            }
            if !shells.iter().any(|s| s == sh) {
                shells.push(sh.to_vec());
            }
        }
        let mut out = Vec::new();
        for s in &shells {
            out.extend_from_slice(s);
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shells_path(), &out) {
            eprintln!(
                "add-shell: cannot update {}: {}",
                shells_path().display(),
                e
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct RemoveShellApplet;
impl Applet for RemoveShellApplet {
    fn name(&self) -> &'static str {
        "remove-shell"
    }
    fn description(&self) -> &'static str {
        "Remove a shell from /etc/shells"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("remove-shell", "SHELL...", "Remove SHELLs from /etc/shells");
            }
        }
        if args.is_empty() {
            eprintln!("Usage: remove-shell SHELL...");
            return Ok(1);
        }
        let shells = read_shells();
        let mut missing = false;
        for a in args {
            let sh = a.as_bytes();
            if !shells.iter().any(|s| s == sh) {
                eprintln!(
                    "remove-shell: '{}' not in {}",
                    String::from_utf8_lossy(sh),
                    shells_path().display()
                );
                missing = true;
            }
        }
        if missing {
            return Ok(1);
        }
        let gone: Vec<&[u8]> = args.iter().map(|a| a.as_bytes()).collect();
        let mut out = Vec::new();
        for s in &shells {
            if !gone.contains(&s.as_slice()) {
                out.extend_from_slice(s);
                out.push(b'\n');
            }
        }
        if let Err(e) = atomic_replace(&shells_path(), &out) {
            eprintln!(
                "remove-shell: cannot update {}: {}",
                shells_path().display(),
                e
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct VlockApplet;
impl Applet for VlockApplet {
    fn name(&self) -> &'static str {
        "vlock"
    }
    fn description(&self) -> &'static str {
        "Lock the terminal until the session password is entered"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("vlock", "[-a]", "Lock the terminal (current tty subset)");
            } else if a.as_bytes() == b"-a" {
            } else {
                eprintln!(
                    "vlock: invalid option '{}'",
                    String::from_utf8_lossy(a.as_bytes())
                );
                return Ok(1);
            }
        }
        let p1 = match read_secret(b"Password: ", false) {
            Some(p) if !p.is_empty() => p,
            _ => {
                eprintln!("vlock: empty password, not locking");
                return Ok(1);
            }
        };
        let p2 = match read_secret(b"Password (again): ", false) {
            Some(p) => p,
            None => {
                eprintln!("vlock: input error");
                return Ok(1);
            }
        };
        if p1 != p2 {
            eprintln!("vlock: passwords do not match");
            return Ok(1);
        }
        eprintln!("vlock: terminal locked, enter password to unlock");
        loop {
            match read_secret(b"Password: ", false) {
                Some(p) if p == p1 => break,
                Some(_) => eprintln!("vlock: incorrect password"),
                None => {
                    eprintln!("vlock: input closed while locked");
                    return Ok(1);
                }
            }
        }
        Ok(0)
    }
}

pub struct NologinApplet;
impl Applet for NologinApplet {
    fn name(&self) -> &'static str {
        "nologin"
    }
    fn description(&self) -> &'static str {
        "Refuse login (politely)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if let Some(a) = args.first() {
            if is_help(a) {
                return help_out("nologin", "", "Print the nologin message and exit 1");
            }
            eprintln!(
                "nologin: invalid option '{}'",
                String::from_utf8_lossy(a.as_bytes())
            );
            return Ok(1);
        }
        let txt = acct_path("BB_NOLOGIN_TXT", "/etc/nologin.txt");
        match fs::read(&txt) {
            Ok(d) if !d.is_empty() => {
                let stdout = io::stdout();
                let mut o = stdout.lock();
                o.write_all(&d)?;
                if !d.ends_with(b"\n") {
                    o.write_all(b"\n")?;
                }
            }
            _ => {
                eprintln!("This account is currently not available.");
            }
        }
        Ok(1)
    }
}
