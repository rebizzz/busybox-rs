use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

// --- shared tiny helpers (one octal parser, one uid:gid splitter) ---

fn parse_octal(b: &[u8]) -> Option<u32> {
    if b.is_empty() || b.len() > 5 {
        return None;
    }
    let mut v: u32 = 0;
    for &c in b {
        if !(b'0'..=b'7').contains(&c) {
            return None;
        }
        v = v * 8 + (c - b'0') as u32;
        if v > 0o7777 {
            return None;
        }
    }
    Some(v)
}

fn split_user_group(spec: &[u8]) -> (&[u8], Option<&[u8]>) {
    match spec.iter().position(|&c| c == b':' || c == b'.') {
        Some(i) => (&spec[..i], Some(&spec[i + 1..])),
        None => (spec, None),
    }
}

fn parse_id_num(b: &[u8]) -> Option<u32> {
    if b.is_empty() {
        return None;
    }
    let mut v: u32 = 0;
    for &c in b {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v.checked_mul(10)?.checked_add((c - b'0') as u32)?;
    }
    Some(v)
}

fn resolve_uid(name: &[u8]) -> Option<u32> {
    if let Some(n) = parse_id_num(name) {
        return Some(n);
    }
    let c = CString::new(name).ok()?;
    let p = unsafe { libc::getpwnam(c.as_ptr()) };
    if p.is_null() {
        None
    } else {
        Some(unsafe { (*p).pw_uid })
    }
}

fn resolve_gid(name: &[u8]) -> Option<u32> {
    if let Some(n) = parse_id_num(name) {
        return Some(n);
    }
    let c = CString::new(name).ok()?;
    let p = unsafe { libc::getgrnam(c.as_ptr()) };
    if p.is_null() {
        None
    } else {
        Some(unsafe { (*p).gr_gid })
    }
}

fn uid_name(uid: u32) -> Vec<u8> {
    let p = unsafe { libc::getpwuid(uid) };
    if p.is_null() {
        let mut b = Vec::new();
        put_num(&mut b, uid as u64);
        return b;
    }
    unsafe { CStr::from_ptr((*p).pw_name) }.to_bytes().to_vec()
}

fn gid_name(gid: u32) -> Vec<u8> {
    let p = unsafe { libc::getgrgid(gid) };
    if p.is_null() {
        let mut b = Vec::new();
        put_num(&mut b, gid as u64);
        return b;
    }
    unsafe { CStr::from_ptr((*p).gr_name) }.to_bytes().to_vec()
}

fn path_cstr(p: &Path) -> Option<CString> {
    CString::new(p.as_os_str().as_bytes()).ok()
}

fn lstat_of(p: &Path, follow: bool) -> Option<libc::stat> {
    let c = path_cstr(p)?;
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let r = unsafe {
        if follow {
            libc::stat(c.as_ptr(), &mut st)
        } else {
            libc::lstat(c.as_ptr(), &mut st)
        }
    };
    if r != 0 {
        None
    } else {
        Some(st)
    }
}

fn put_num(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    let mut t = [0u8; 20];
    let mut i = t.len();
    while n > 0 {
        i -= 1;
        t[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&t[i..]);
}

fn put_oct(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    let mut t = [0u8; 22];
    let mut i = t.len();
    while n > 0 {
        i -= 1;
        t[i] = b'0' + (n % 8) as u8;
        n /= 8;
    }
    buf.extend_from_slice(&t[i..]);
}

fn put_human(buf: &mut Vec<u8>, bytes: u64) {
    const U: [u8; 6] = *b"KMGTPE";
    if bytes < 1024 {
        put_num(buf, bytes);
        return;
    }
    let mut div = 1024u64;
    let mut ui = 0usize;
    while bytes / div >= 1024 && ui + 1 < U.len() {
        div *= 1024;
        ui += 1;
    }
    put_num(buf, bytes / div);
    buf.push(b'.');
    buf.push(b'0' + ((bytes % div) * 10 / div) as u8);
    buf.push(U[ui]);
}

fn put_mode_str(buf: &mut Vec<u8>, mode: u32) {
    const T: &[u8] = b"rwxrwxrwx";
    for (i, &c) in T.iter().enumerate() {
        let bit = 1 << (8 - i);
        buf.push(if mode & bit != 0 { c } else { b'-' });
    }
    if mode & 0o4000 != 0 {
        buf[3] = if buf[3] == b'x' { b's' } else { b'S' };
    }
    if mode & 0o2000 != 0 {
        buf[6] = if buf[6] == b'x' { b's' } else { b'S' };
    }
    if mode & 0o1000 != 0 {
        buf[9] = if buf[9] == b'x' { b't' } else { b'T' };
    }
}

fn file_type_char(mode: u32) -> u8 {
    match mode & libc::S_IFMT {
        libc::S_IFDIR => b'd',
        libc::S_IFLNK => b'l',
        libc::S_IFCHR => b'c',
        libc::S_IFBLK => b'b',
        libc::S_IFIFO => b'p',
        libc::S_IFSOCK => b's',
        _ => b'-',
    }
}

fn do_chmod(p: &Path, mode: u32) -> io::Result<()> {
    let c = path_cstr(p).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "bad path"))?;
    if unsafe { libc::chmod(c.as_ptr(), mode) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn chmod_tree(p: &Path, mode: u32) -> i32 {
    let mut rc = 0;
    if let Err(e) = do_chmod(p, mode) {
        eprintln!("chmod: {}: {}", p.display(), e);
        rc = 1;
    }
    if let Ok(rd) = fs::read_dir(p) {
        for e in rd.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let sub = e.path();
                let r = chmod_tree(&sub, mode);
                if r != 0 {
                    rc = 1;
                }
            } else if let Err(er) = do_chmod(&e.path(), mode) {
                eprintln!("chmod: {}: {}", e.path().display(), er);
                rc = 1;
            }
        }
    }
    rc
}

fn do_chown(p: &Path, uid: u32, gid: u32, noderef: bool) -> io::Result<()> {
    let c = path_cstr(p).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "bad path"))?;
    let r = unsafe {
        if noderef {
            libc::lchown(c.as_ptr(), uid, gid)
        } else {
            libc::chown(c.as_ptr(), uid, gid)
        }
    };
    if r != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn chown_tree(p: &Path, uid: u32, gid: u32, noderef: bool) -> i32 {
    let mut rc = 0;
    if let Err(e) = do_chown(p, uid, gid, noderef) {
        eprintln!("chown: {}: {}", p.display(), e);
        rc = 1;
    }
    let descend = if noderef {
        fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false)
    } else {
        p.is_dir()
    };
    if descend {
        if let Ok(rd) = fs::read_dir(p) {
            for e in rd.flatten() {
                if chown_tree(&e.path(), uid, gid, noderef) != 0 {
                    rc = 1;
                }
            }
        }
    }
    rc
}

// --- chmod ---

pub struct ChmodApplet;
impl Applet for ChmodApplet {
    fn name(&self) -> &'static str {
        "chmod"
    }
    fn description(&self) -> &'static str {
        "Change file mode bits"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut pos: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            let b = a.as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(r);
                }
                break;
            } else if b.starts_with(b"-") && b.len() > 1 {
                for &c in &b[1..] {
                    if c == b'R' {
                        recursive = true;
                    } else {
                        eprintln!("chmod: invalid option -- '{}'", c as char);
                        return Ok(1);
                    }
                }
            } else {
                pos.push(a);
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("chmod: missing operand");
            return Ok(1);
        }
        let mode = match parse_octal(pos[0].as_bytes()) {
            Some(m) => m,
            None => {
                eprintln!("chmod: invalid mode '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            if recursive && fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false) {
                if chmod_tree(p, mode) != 0 {
                    rc = 1;
                }
            } else if let Err(e) = do_chmod(p, mode) {
                eprintln!("chmod: {}: {}", p.display(), e);
                rc = 1;
            }
        }
        Ok(rc)
    }
}

// --- chown ---

pub struct ChownApplet;
impl Applet for ChownApplet {
    fn name(&self) -> &'static str {
        "chown"
    }
    fn description(&self) -> &'static str {
        "Change file owner and group"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut noderef = false;
        let mut pos: Vec<&OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'R' => recursive = true,
                        b'h' => noderef = true,
                        _ => {
                            eprintln!("chown: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(a);
            }
        }
        if pos.is_empty() {
            eprintln!("chown: missing operand");
            return Ok(1);
        }
        if pos.len() < 2 {
            eprintln!(
                "chown: missing file operand after '{}'",
                pos[0].to_string_lossy()
            );
            return Ok(1);
        }
        let (u, g) = split_user_group(pos[0].as_bytes());
        if u.is_empty() && g.map(|x| x.is_empty()).unwrap_or(true) {
            eprintln!("chown: invalid owner");
            return Ok(1);
        }
        let uid = if u.is_empty() {
            u32::MAX
        } else {
            match resolve_uid(u) {
                Some(v) => v,
                None => {
                    eprintln!("chown: invalid user '{}'", String::from_utf8_lossy(u));
                    return Ok(1);
                }
            }
        };
        let gid = match g {
            Some(gr) if !gr.is_empty() => match resolve_gid(gr) {
                Some(v) => v,
                None => {
                    eprintln!("chown: invalid group '{}'", String::from_utf8_lossy(gr));
                    return Ok(1);
                }
            },
            _ => u32::MAX,
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            let r = if recursive {
                chown_tree(p, uid, gid, noderef)
            } else {
                match do_chown(p, uid, gid, noderef) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("chown: {}: {}", p.display(), e);
                        1
                    }
                }
            };
            if r != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}

// --- chgrp ---

pub struct ChgrpApplet;
impl Applet for ChgrpApplet {
    fn name(&self) -> &'static str {
        "chgrp"
    }
    fn description(&self) -> &'static str {
        "Change file group"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut noderef = false;
        let mut pos: Vec<&OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'R' => recursive = true,
                        b'h' => noderef = true,
                        _ => {
                            eprintln!("chgrp: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(a);
            }
        }
        if pos.len() < 2 {
            eprintln!("chgrp: missing operand");
            return Ok(1);
        }
        let gid = match resolve_gid(pos[0].as_bytes()) {
            Some(v) => v,
            None => {
                eprintln!("chgrp: invalid group '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            let r = if recursive {
                chown_tree(p, u32::MAX, gid, noderef)
            } else {
                match do_chown(p, u32::MAX, gid, noderef) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("chgrp: {}: {}", p.display(), e);
                        1
                    }
                }
            };
            if r != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}

// --- ln ---

pub struct LnApplet;
impl Applet for LnApplet {
    fn name(&self) -> &'static str {
        "ln"
    }
    fn description(&self) -> &'static str {
        "Make links between files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sym = false;
        let mut force = false;
        let mut verbose = false;
        let mut no_target_dir = false;
        let mut backup = false;
        let mut pos: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(Path::new(r));
                }
                break;
            } else if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b's' => sym = true,
                        b'f' => force = true,
                        b'v' => verbose = true,
                        b'T' => no_target_dir = true,
                        b'n' => no_target_dir = true,
                        b'b' => backup = true,
                        b'S' => {
                            backup = true;
                            if j + 1 >= b.len() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("ln: option requires an argument -- 'S'");
                                    return Ok(1);
                                }
                            }
                            break;
                        }
                        _ => {
                            eprintln!("ln: invalid option -- '{}'", b[j] as char);
                            return Ok(1);
                        }
                    }
                    j += 1;
                }
            } else {
                pos.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("ln: missing file operand");
            return Ok(1);
        }
        let (srcs, dst) = (pos[..pos.len() - 1].to_vec(), pos[pos.len() - 1]);
        let dst_is_dir = if no_target_dir {
            false
        } else {
            fs::symlink_metadata(dst)
                .map(|m| m.is_dir())
                .unwrap_or(false)
        };
        if srcs.len() > 1 && !dst_is_dir {
            eprintln!("ln: target '{}' is not a directory", dst.display());
            return Ok(1);
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        ln_all(
            &mut out, &srcs, dst, dst_is_dir, sym, backup, force, verbose,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn ln_all(
    out: &mut impl Write,
    srcs: &[&Path],
    dst: &Path,
    dst_is_dir: bool,
    sym: bool,
    backup: bool,
    force: bool,
    verbose: bool,
) -> Result<i32> {
    let mut rc = 0;
    for s in srcs {
        let target = if dst_is_dir {
            dst.join(s.file_name().unwrap_or(s.as_os_str()))
        } else {
            dst.to_path_buf()
        };
        if backup && target.exists() {
            let mut bkp = target.as_os_str().as_bytes().to_vec();
            bkp.push(b'~');
            let _ = fs::rename(&target, Path::new(OsStr::from_bytes(&bkp)));
        } else if force {
            let _ = fs::remove_file(&target);
        }
        let r = if sym {
            std::os::unix::fs::symlink(s, &target)
        } else {
            fs::hard_link(s, &target)
        };
        if let Err(e) = r {
            eprintln!("ln: failed to create link '{}': {}", target.display(), e);
            rc = 1;
        } else if verbose {
            out.write_all(target.as_os_str().as_bytes())?;
            out.write_all(b" -> ")?;
            out.write_all(s.as_os_str().as_bytes())?;
            out.write_all(b"\n")?;
        }
    }
    Ok(rc)
}

// --- stat ---

pub struct StatApplet;
impl Applet for StatApplet {
    fn name(&self) -> &'static str {
        "stat"
    }
    fn description(&self) -> &'static str {
        "Display file or filesystem status"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fmt: Option<&[u8]> = None;
        let mut fs_mode = false;
        let mut follow = false;
        let mut terse = false;
        let mut pos: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(Path::new(r));
                }
                break;
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("stat: option requires an argument -- 'c'");
                    return Ok(1);
                }
                fmt = Some(args[i].as_bytes());
            } else if b.starts_with(b"-c") && b.len() > 2 {
                fmt = Some(&args[i].as_bytes()[2..]);
            } else if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'f' => fs_mode = true,
                        b'L' => follow = true,
                        b't' => terse = true,
                        _ => {
                            eprintln!("stat: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if pos.is_empty() {
            eprintln!("stat: missing operand");
            return Ok(1);
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for p in pos {
            if emit_stat(&mut out, p, fmt, fs_mode, follow, terse)? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}

fn emit_stat(
    out: &mut impl Write,
    p: &Path,
    fmt: Option<&[u8]>,
    fs_mode: bool,
    follow: bool,
    terse: bool,
) -> Result<i32> {
    if fs_mode {
        match fs_stat_line(p) {
            Some(line) => {
                out.write_all(&line)?;
                out.write_all(b"\n")?;
                return Ok(0);
            }
            None => {
                eprintln!("stat: cannot stat '{}'", p.display());
                return Ok(1);
            }
        }
    }
    let st = match lstat_of(p, follow) {
        Some(s) => s,
        None => {
            eprintln!("stat: cannot stat '{}'", p.display());
            return Ok(1);
        }
    };
    if let Some(f) = fmt {
        let mut line = Vec::new();
        stat_format(&mut line, f, p, &st);
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    } else if terse {
        let mut line = Vec::new();
        line.extend_from_slice(p.as_os_str().as_bytes());
        line.push(b' ');
        put_num(&mut line, st.st_size as u64);
        line.push(b' ');
        put_num(&mut line, st.st_blocks as u64);
        line.push(b' ');
        put_num(&mut line, (st.st_mode & 0o7777) as u64);
        line.push(b' ');
        put_num(&mut line, st.st_uid as u64);
        line.push(b' ');
        put_num(&mut line, st.st_gid as u64);
        line.push(b' ');
        put_num(&mut line, st.st_ino);
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    } else {
        let mut line = Vec::new();
        line.extend_from_slice(b"  File: ");
        line.extend_from_slice(p.as_os_str().as_bytes());
        line.extend_from_slice(b"\n  Size: ");
        put_num(&mut line, st.st_size as u64);
        line.extend_from_slice(b"  Blocks: ");
        put_num(&mut line, st.st_blocks as u64);
        line.extend_from_slice(b"\nAccess: (");
        let mut oct = Vec::new();
        put_num(&mut oct, (st.st_mode & 0o7777) as u64);
        line.extend_from_slice(&oct);
        line.extend_from_slice(b"/");
        line.push(file_type_char(st.st_mode as u32));
        put_mode_str(&mut line, st.st_mode as u32);
        line.extend_from_slice(b")  Uid: (");
        put_num(&mut line, st.st_uid as u64);
        line.extend_from_slice(b"/");
        line.extend_from_slice(&uid_name(st.st_uid));
        line.extend_from_slice(b")  Gid: (");
        put_num(&mut line, st.st_gid as u64);
        line.extend_from_slice(b"/");
        line.extend_from_slice(&gid_name(st.st_gid));
        line.extend_from_slice(b")");
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    }
    Ok(0)
}

fn stat_format(buf: &mut Vec<u8>, fmt: &[u8], p: &Path, st: &libc::stat) {
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' || i + 1 >= fmt.len() {
            buf.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        match fmt[i] {
            b'a' => put_oct(buf, (st.st_mode & 0o7777) as u64),
            b'A' => {
                buf.push(file_type_char(st.st_mode));
                put_mode_str(buf, st.st_mode);
            }
            b's' => put_num(buf, st.st_size as u64),
            b'b' => put_num(buf, st.st_blocks as u64),
            b'B' => put_num(buf, 512),
            b'h' => put_num(buf, st.st_nlink),
            b'u' => put_num(buf, st.st_uid as u64),
            b'g' => put_num(buf, st.st_gid as u64),
            b'U' => buf.extend_from_slice(&uid_name(st.st_uid)),
            b'G' => buf.extend_from_slice(&gid_name(st.st_gid)),
            b'n' => buf.extend_from_slice(p.as_os_str().as_bytes()),
            b'F' => buf.extend_from_slice(match st.st_mode & libc::S_IFMT {
                libc::S_IFDIR => b"directory",
                libc::S_IFLNK => b"symbolic link",
                libc::S_IFCHR => b"character device",
                libc::S_IFBLK => b"block device",
                libc::S_IFIFO => b"fifo",
                libc::S_IFSOCK => b"socket",
                _ => b"regular file",
            }),
            b'i' => put_num(buf, st.st_ino),
            b'd' => put_num(buf, st.st_dev),
            _ => {
                buf.push(b'%');
                buf.push(fmt[i]);
            }
        }
        i += 1;
    }
}

fn fs_stat_line(p: &Path) -> Option<Vec<u8>> {
    let c = path_cstr(p)?;
    let mut v: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut v) } != 0 {
        return None;
    }
    let mut b = Vec::new();
    b.extend_from_slice(p.as_os_str().as_bytes());
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_bsize as u64);
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_blocks);
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_bavail);
    Some(b)
}

// --- du ---

fn du_bytes(p: &Path, follow: bool, apparent: bool, one_fs: bool, top_dev: u64) -> u64 {
    let st = match lstat_of(p, follow) {
        Some(s) => s,
        None => return 0,
    };
    if one_fs && st.st_dev != top_dev {
        return 0;
    }
    let here = if apparent {
        st.st_size.max(0) as u64
    } else {
        (st.st_blocks.max(0) as u64) * 512
    };
    if (st.st_mode & libc::S_IFMT) != libc::S_IFDIR {
        return here;
    }
    let mut sum = here;
    if let Ok(rd) = fs::read_dir(p) {
        for e in rd.flatten() {
            sum += du_bytes(&e.path(), follow, apparent, one_fs, top_dev);
        }
    }
    sum
}

#[derive(Default)]
struct DuOpts {
    summarize: bool,
    all: bool,
    human: bool,
    apparent: bool,
    total: bool,
    one_fs: bool,
    follow: bool,
    div: u64,
    maxdepth: Option<u32>,
}

fn parse_depth(v: &[u8]) -> Option<u32> {
    std::str::from_utf8(v)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
}

fn du_parse<'a>(args: &'a [OsString], o: &mut DuOpts, pos: &mut Vec<&'a Path>) -> Result<i32> {
    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"--" {
            for r in &args[i + 1..] {
                pos.push(Path::new(r));
            }
            break;
        } else if b == b"-d" || b.starts_with(b"--max-depth=") {
            let v = if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("du: option requires an argument -- 'd'");
                    return Ok(1);
                }
                args[i].as_bytes().to_vec()
            } else {
                b[b"--max-depth=".len()..].to_vec()
            };
            match parse_depth(&v) {
                Some(n) => o.maxdepth = Some(n),
                None => {
                    eprintln!("du: invalid max depth");
                    return Ok(1);
                }
            }
        } else if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
            let mut j = 1;
            while j < b.len() {
                match b[j] {
                    b's' => o.summarize = true,
                    b'a' => o.all = true,
                    b'h' => o.human = true,
                    b'k' => o.div = 1024,
                    b'm' => o.div = 1024 * 1024,
                    b'b' => o.apparent = true,
                    b'c' => o.total = true,
                    b'x' => o.one_fs = true,
                    b'L' => o.follow = true,
                    b'H' => o.follow = true,
                    b'd' => {
                        let rest = &b[j + 1..];
                        let v: Vec<u8> = if rest.is_empty() {
                            i += 1;
                            if i >= args.len() {
                                eprintln!("du: option requires an argument -- 'd'");
                                return Ok(1);
                            }
                            args[i].as_bytes().to_vec()
                        } else {
                            rest.to_vec()
                        };
                        match parse_depth(&v) {
                            Some(n) => o.maxdepth = Some(n),
                            None => {
                                eprintln!("du: invalid max depth");
                                return Ok(1);
                            }
                        }
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
        } else {
            pos.push(Path::new(&args[i]));
        }
        i += 1;
    }
    Ok(0)
}

pub struct DuApplet;
impl Applet for DuApplet {
    fn name(&self) -> &'static str {
        "du"
    }
    fn description(&self) -> &'static str {
        "Estimate file space usage"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut o = DuOpts {
            div: 1024,
            ..Default::default()
        };
        let mut pos: Vec<&Path> = Vec::new();
        if du_parse(args, &mut o, &mut pos)? != 0 {
            return Ok(1);
        }
        if pos.is_empty() {
            pos.push(Path::new("."));
        }
        if o.summarize {
            o.maxdepth = Some(0);
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        let mut grand: u64 = 0;
        for p in &pos {
            if lstat_of(p, o.follow).is_none() {
                eprintln!("du: cannot access '{}'", p.display());
                rc = 1;
                continue;
            }
            let dev = lstat_of(p, o.follow).map(|s| s.st_dev).unwrap_or(0);
            grand += du_print(
                &mut out, p, o.follow, o.apparent, o.one_fs, dev, o.human, o.div, o.all,
                o.maxdepth, 0,
            )?;
        }
        if o.total {
            let mut line = Vec::new();
            if o.human {
                put_human(&mut line, grand);
            } else if o.apparent {
                put_num(&mut line, grand);
            } else {
                put_num(&mut line, grand.div_ceil(o.div));
            }
            line.extend_from_slice(b"\ttotal\n");
            out.write_all(&line)?;
        }
        Ok(rc)
    }
}

#[allow(clippy::too_many_arguments)]
fn du_print(
    out: &mut impl Write,
    p: &Path,
    follow: bool,
    apparent: bool,
    one_fs: bool,
    top_dev: u64,
    human: bool,
    div: u64,
    all: bool,
    maxdepth: Option<u32>,
    depth: u32,
) -> Result<u64> {
    let bytes = du_bytes(p, follow, apparent, one_fs, top_dev);
    let st = lstat_of(p, follow);
    let is_dir = st
        .map(|s| (s.st_mode & libc::S_IFMT) == libc::S_IFDIR)
        .unwrap_or(false);
    let show = all || is_dir || depth == 0;
    let within = maxdepth.map(|m| depth <= m).unwrap_or(true);
    if show && within {
        let mut line = Vec::new();
        if human {
            put_human(&mut line, bytes);
        } else if apparent {
            put_num(&mut line, bytes);
        } else {
            put_num(&mut line, bytes.div_ceil(div));
        }
        line.push(b'\t');
        line.extend_from_slice(p.as_os_str().as_bytes());
        line.push(b'\n');
        out.write_all(&line)?;
    }
    if is_dir && maxdepth.map(|m| depth < m).unwrap_or(true) {
        if let Ok(rd) = fs::read_dir(p) {
            let mut names: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            names.sort();
            for c in names {
                let stc = lstat_of(&c, follow);
                let cd = stc
                    .map(|s| (s.st_mode & libc::S_IFMT) == libc::S_IFDIR)
                    .unwrap_or(false);
                if cd || all {
                    du_print(
                        out,
                        &c,
                        follow,
                        apparent,
                        one_fs,
                        top_dev,
                        human,
                        div,
                        all,
                        maxdepth,
                        depth + 1,
                    )?;
                }
            }
        }
    }
    Ok(bytes)
}

// --- df ---

pub struct DfApplet;
impl Applet for DfApplet {
    fn name(&self) -> &'static str {
        "df"
    }
    fn description(&self) -> &'static str {
        "Report filesystem disk space usage"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut human = false;
        let mut div: u64 = 1024;
        let mut inodes = false;
        let mut pos: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'h' => human = true,
                        b'k' => div = 1024,
                        b'm' => div = 1024 * 1024,
                        b'P' => {}
                        b'i' => inodes = true,
                        b'a' => {}
                        _ => {}
                    }
                }
            } else {
                pos.push(Path::new(a));
            }
        }
        let all_mounts = read_mounts();
        let mounts: Vec<(Vec<u8>, Vec<u8>)> = if pos.is_empty() {
            all_mounts
        } else {
            pos.iter()
                .map(|p| {
                    let pb = p.as_os_str().as_bytes();
                    let dev = find_dev(&all_mounts, pb).unwrap_or(b"-").to_vec();
                    (dev, pb.to_vec())
                })
                .collect()
        };
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut head = Vec::new();
        head.extend_from_slice(b"Filesystem ");
        head.extend_from_slice(if inodes {
            b"Inodes IUsed IFree IUse% "
        } else if human {
            b"Size Used Avail Use% "
        } else {
            b"1K-blocks Used Available Use% "
        });
        head.extend_from_slice(b"Mounted on\n");
        out.write_all(&head)?;
        let mut rc = 0;
        for (dev, mnt) in &mounts {
            let target = if mnt == b"-" { dev } else { mnt };
            let p = Path::new(OsStr::from_bytes(target));
            let c = match path_cstr(p) {
                Some(v) => v,
                None => {
                    rc = 1;
                    continue;
                }
            };
            let mut v: libc::statvfs = unsafe { std::mem::zeroed() };
            if unsafe { libc::statvfs(c.as_ptr(), &mut v) } != 0 {
                eprintln!("df: {}: {}", p.display(), io::Error::last_os_error());
                rc = 1;
                continue;
            }
            let mut line = Vec::new();
            line.extend_from_slice(dev);
            line.push(b' ');
            df_row(&mut line, &v, human, div, inodes);
            line.push(b' ');
            line.extend_from_slice(if mnt == b"-" { target } else { mnt });
            line.push(b'\n');
            out.write_all(&line)?;
        }
        Ok(rc)
    }
}

fn df_row(line: &mut Vec<u8>, v: &libc::statvfs, human: bool, div: u64, inodes: bool) {
    if inodes {
        put_num(line, v.f_files);
        line.push(b' ');
        put_num(line, v.f_files - v.f_ffree);
        line.push(b' ');
        put_num(line, v.f_ffree);
        line.push(b' ');
        let pct = (v.f_files - v.f_ffree)
            .saturating_mul(100)
            .checked_div(v.f_files)
            .unwrap_or(0);
        put_num(line, pct);
        line.push(b'%');
        return;
    }
    let bsize = v.f_bsize;
    let tot = v.f_blocks * bsize;
    let free = v.f_bavail * bsize;
    let used = tot.saturating_sub(v.f_bfree * bsize);
    if human {
        put_human(line, tot);
        line.push(b' ');
        put_human(line, used);
        line.push(b' ');
        put_human(line, free);
    } else {
        put_num(line, tot / div);
        line.push(b' ');
        put_num(line, used / div);
        line.push(b' ');
        put_num(line, free / div);
    }
    line.push(b' ');
    let pct = used.saturating_mul(100).checked_div(tot).unwrap_or(0);
    put_num(line, pct);
    line.push(b'%');
}

fn find_dev<'a>(mounts: &'a [(Vec<u8>, Vec<u8>)], path: &[u8]) -> Option<&'a [u8]> {
    // Longest mount-point prefix wins (path and mnt are absolute here; also try raw).
    let mut best: Option<&'a [u8]> = None;
    let mut best_len = 0;
    for (dev, mnt) in mounts {
        let hit = path == mnt
            || (path.starts_with(mnt.as_slice())
                && (mnt == b"/" || path.get(mnt.len()) == Some(&b'/')));
        if hit && mnt.len() >= best_len {
            best_len = mnt.len();
            best = Some(dev);
        }
    }
    best
}

fn read_mounts() -> Vec<(Vec<u8>, Vec<u8>)> {
    let data = fs::read("/proc/mounts").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        let mut parts = line.split(|&b| b == b' ');
        let dev = parts.next().unwrap_or(b"").to_vec();
        let mnt = parts.next().unwrap_or(b"").to_vec();
        if dev.is_empty() || mnt.is_empty() {
            continue;
        }
        out.push((dev, mnt));
    }
    if out.is_empty() {
        out.push((b"/dev/root".to_vec(), b"/".to_vec()));
    }
    out
}
