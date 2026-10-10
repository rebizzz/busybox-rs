use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::FromRawFd;
use std::path::Path;
use std::process::Command;

fn path_cstr(p: &Path) -> Option<CString> {
    CString::new(p.as_os_str().as_bytes()).ok()
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

    let whole = bytes / div;
    let frac = (bytes % div) * 10 / div;
    put_num(buf, whole);
    buf.push(b'.');
    buf.push(b'0' + frac as u8);
    buf.push(U[ui]);
}

fn read_mounts() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let data = fs::read("/proc/mounts").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(|&b| b == b' ');
        let dev = parts.next().unwrap_or(b"").to_vec();
        let mnt = parts.next().unwrap_or(b"").to_vec();
        let fs = parts.next().unwrap_or(b"").to_vec();
        if dev.is_empty() || mnt.is_empty() {
            continue;
        }
        out.push((dev, mnt, fs));
    }
    out
}

#[derive(Clone, Default)]
struct FstabEntry {
    dev: Vec<u8>,
    mnt: Vec<u8>,
    fstype: Vec<u8>,
    opts: Vec<u8>,
}

fn read_fstab(path: &Path) -> Vec<FstabEntry> {
    let data = fs::read(path).unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        let line = if let Some(i) = line.iter().position(|&b| b == b'#') {
            &line[..i]
        } else {
            line
        };
        let f: Vec<&[u8]> = line
            .split(|&b| b == b' ' || b == b'\t')
            .filter(|s| !s.is_empty())
            .collect();
        if f.len() < 4 {
            continue;
        }
        out.push(FstabEntry {
            dev: f[0].to_vec(),
            mnt: f[1].to_vec(),
            fstype: f[2].to_vec(),
            opts: f[3].to_vec(),
        });
    }
    out
}

fn fold_opt(name: &[u8], flags: &mut libc::c_ulong, data: &mut Vec<Vec<u8>>) {
    match name {
        b"ro" => *flags |= libc::MS_RDONLY,
        b"rw" => *flags &= !libc::MS_RDONLY,
        b"sync" => *flags |= libc::MS_SYNCHRONOUS,
        b"async" => *flags &= !libc::MS_SYNCHRONOUS,
        b"atime" => *flags &= !libc::MS_NOATIME,
        b"noatime" => *flags |= libc::MS_NOATIME,
        b"dev" => *flags &= !libc::MS_NODEV,
        b"nodev" => *flags |= libc::MS_NODEV,
        b"exec" => *flags &= !libc::MS_NOEXEC,
        b"noexec" => *flags |= libc::MS_NOEXEC,
        b"suid" => *flags &= !libc::MS_NOSUID,
        b"nosuid" => *flags |= libc::MS_NOSUID,
        b"relatime" => *flags |= libc::MS_RELATIME,
        b"norelatime" => *flags &= !libc::MS_RELATIME,
        b"remount" => *flags |= libc::MS_REMOUNT,
        b"bind" => *flags |= libc::MS_BIND,
        b"move" => *flags |= libc::MS_MOVE,
        b"defaults" | b"loop" | b"noauto" | b"auto" => {}
        _ => data.push(name.to_vec()),
    }
}

fn proc_filesystems() -> Vec<Vec<u8>> {
    let data = fs::read("/proc/filesystems").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        let mut parts = line
            .split(|&b| b == b'\t' || b == b' ')
            .filter(|s| !s.is_empty());
        let first = parts.next().unwrap_or(b"");
        let name = if first == b"nodev" {
            parts.next().unwrap_or(b"")
        } else {
            first
        };
        if !name.is_empty() {
            out.push(name.to_vec());
        }
    }
    out
}

fn mount_one(
    dev: &[u8],
    mnt: &[u8],
    fstype: &[u8],
    o_opts: &[Vec<u8>],
    rdonly: bool,
    fake: bool,
    verbose: bool,
) -> i32 {
    let mut flags: libc::c_ulong = 0;
    let mut data: Vec<Vec<u8>> = Vec::new();
    if rdonly {
        flags |= libc::MS_RDONLY;
    }
    for o in o_opts {
        for name in o.split(|&b| b == b',') {
            if !name.is_empty() {
                fold_opt(name, &mut flags, &mut data);
            }
        }
    }
    let types: Vec<Vec<u8>> = if fstype == b"auto" || fstype.is_empty() {
        vec![b"".to_vec()]
    } else {
        fstype.split(|&b| b == b',').map(|s| s.to_vec()).collect()
    };
    let dev_p = Path::new(std::ffi::OsStr::from_bytes(dev));
    let mnt_p = Path::new(std::ffi::OsStr::from_bytes(mnt));
    let dev_c = match path_cstr(dev_p) {
        Some(c) => c,
        None => {
            eprintln!("mount: bad device name");
            return 1;
        }
    };
    let mnt_c = match path_cstr(mnt_p) {
        Some(c) => c,
        None => {
            eprintln!("mount: bad mount point");
            return 1;
        }
    };
    let data_c = if data.is_empty() {
        None
    } else {
        let joined = data.join(&b',');
        match CString::new(joined) {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!("mount: bad -o option");
                return 1;
            }
        }
    };
    let data_ptr = data_c
        .as_ref()
        .map(|c| c.as_ptr().cast())
        .unwrap_or(std::ptr::null());
    let show_line = || {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = out.write_all(dev);
        let _ = out.write_all(b" on ");
        let _ = out.write_all(mnt);
        let _ = out.write_all(b" type ");
        let _ = out.write_all(if fstype.is_empty() { b"auto" } else { fstype });
        let _ = out.write_all(b"\n");
    };
    if fake {
        show_line();
        return 0;
    }
    if verbose {
        show_line();
    }

    let candidates: Vec<Vec<u8>> = if types.len() == 1 && types[0].is_empty() {
        proc_filesystems()
    } else {
        types
    };
    if candidates.is_empty() {
        eprintln!("mount: no filesystem types");
        return 1;
    }
    for t in &candidates {
        let t_c = match CString::new(t.clone()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let r = unsafe {
            libc::mount(
                dev_c.as_ptr(),
                mnt_c.as_ptr(),
                t_c.as_ptr(),
                flags,
                data_ptr,
            )
        };
        if r == 0 {
            return 0;
        }
    }
    eprintln!("mount: mounting failed: {}", io::Error::last_os_error());
    1
}

pub struct MountApplet;
impl Applet for MountApplet {
    fn name(&self) -> &'static str {
        "mount"
    }
    fn description(&self) -> &'static str {
        "Mount a filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut all = false;
        let mut fake = false;
        let mut verbose = false;
        let mut rdonly = false;
        let mut fstype: Vec<u8> = Vec::new();
        let mut fstab = Path::new("/etc/fstab").to_path_buf();
        let mut only_opt: Option<Vec<u8>> = None;
        let mut o_opts: Vec<Vec<u8>> = Vec::new();
        let mut pos: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    pos.push(a.as_bytes());
                }
                break;
            } else if b == b"-a" {
                all = true;
            } else if b == b"-f" {
                fake = true;
            } else if b == b"-n" {
            } else if b == b"-v" {
                verbose = true;
            } else if b == b"-r" {
                rdonly = true;
            } else if b == b"-w" {
                rdonly = false;
            } else if b == b"-s" {
            } else if b == b"-t" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mount: option requires an argument -- 't'");
                    return Ok(1);
                }
                fstype = args[i].as_bytes().to_vec();
            } else if b.starts_with(b"-t") && b.len() > 2 {
                fstype = b[2..].to_vec();
            } else if b == b"-T" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mount: option requires an argument -- 'T'");
                    return Ok(1);
                }
                fstab = Path::new(&args[i]).to_path_buf();
            } else if b == b"-O" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mount: option requires an argument -- 'O'");
                    return Ok(1);
                }
                only_opt = Some(args[i].as_bytes().to_vec());
            } else if b == b"-o" {
                i += 1;
                if i >= args.len() {
                    eprintln!("mount: option requires an argument -- 'o'");
                    return Ok(1);
                }
                o_opts.push(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-o") && b.len() > 2 {
                o_opts.push(b[2..].to_vec());
            } else if b.starts_with(b"-") && b.len() > 1 {
                eprintln!("mount: invalid option -- '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b);
            }
            i += 1;
        }
        if all {
            let mut rc = 0;
            for e in read_fstab(&fstab) {
                let has = |o: &[u8]| e.opts.split(|&b| b == b',').any(|x| x == o);
                if has(b"noauto") && only_opt.is_none() {
                    continue;
                }
                if let Some(ref want) = only_opt {
                    if !has(want) {
                        continue;
                    }
                }
                if e.mnt == b"none" || e.mnt == b"swap" {
                    continue;
                }
                let mut opts = o_opts.clone();
                opts.push(e.opts.clone());
                let fs = if fstype.is_empty() {
                    e.fstype.clone()
                } else {
                    fstype.clone()
                };
                rc |= mount_one(&e.dev, &e.mnt, &fs, &opts, rdonly, fake, verbose);
            }
            return Ok(rc);
        }
        if pos.is_empty() {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            for (dev, mnt, fs) in read_mounts() {
                out.write_all(&dev)?;
                out.write_all(b" on ")?;
                out.write_all(&mnt)?;
                out.write_all(b" type ")?;
                out.write_all(&fs)?;
                out.write_all(b"\n")?;
            }
            return Ok(0);
        }
        if pos.len() == 1 {
            let want = pos[0];
            let entries = read_fstab(&fstab);
            for e in &entries {
                if e.dev.as_slice() == want || e.mnt.as_slice() == want {
                    let mut opts = o_opts.clone();
                    opts.push(e.opts.clone());
                    let fs = if fstype.is_empty() {
                        e.fstype.clone()
                    } else {
                        fstype.clone()
                    };
                    return Ok(mount_one(&e.dev, &e.mnt, &fs, &opts, rdonly, fake, verbose));
                }
            }
            eprintln!(
                "mount: can't find '{}' in {}",
                String::from_utf8_lossy(want),
                fstab.display()
            );
            return Ok(1);
        }
        let fs = if fstype.is_empty() {
            b"auto".to_vec()
        } else {
            fstype
        };
        Ok(mount_one(
            pos[0], pos[1], &fs, &o_opts, rdonly, fake, verbose,
        ))
    }
}

fn umount_one(target: &[u8], flags: libc::c_int, retry_ro: bool) -> i32 {
    let p = Path::new(std::ffi::OsStr::from_bytes(target));
    let c = match path_cstr(p) {
        Some(c) => c,
        None => {
            eprintln!("umount: bad path");
            return 1;
        }
    };

    if unsafe { libc::umount2(c.as_ptr(), flags) } == 0 {
        return 0;
    }
    let e = io::Error::last_os_error();
    if retry_ro {
        let r = unsafe {
            libc::mount(
                c.as_ptr(),
                c.as_ptr(),
                std::ptr::null(),
                (libc::MS_REMOUNT | libc::MS_RDONLY) as libc::c_ulong,
                std::ptr::null(),
            )
        };
        if r == 0 {
            return 0;
        }
    }
    eprintln!("umount: {}: {}", String::from_utf8_lossy(target), e);
    1
}

pub struct UmountApplet;
impl Applet for UmountApplet {
    fn name(&self) -> &'static str {
        "umount"
    }
    fn description(&self) -> &'static str {
        "Unmount filesystems"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut all = false;
        let mut retry_ro = false;
        let mut lazy = false;
        let mut force = false;
        let mut types: Vec<u8> = Vec::new();
        let mut pos: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    pos.push(a.as_bytes());
                }
                break;
            } else if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b'a' => all = true,
                        b'r' => retry_ro = true,
                        b'l' => lazy = true,
                        b'f' => force = true,
                        b'n' => {}
                        b'd' => {}
                        b't' => {
                            let rest = &b[j + 1..];
                            if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("umount: option requires an argument -- 't'");
                                    return Ok(1);
                                }
                                types = args[i].as_bytes().to_vec();
                            } else {
                                types = rest.to_vec();
                            }
                            break;
                        }
                        c => {
                            eprintln!("umount: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                    j += 1;
                }
            } else {
                pos.push(b);
            }
            i += 1;
        }
        let mut flags: libc::c_int = 0;
        if lazy {
            flags |= libc::MNT_DETACH;
        }
        if force {
            flags |= libc::MNT_FORCE;
        }
        let type_ok = |fs: &[u8]| types.is_empty() || types.split(|&b| b == b',').any(|t| t == fs);
        if all {
            let mut rc = 0;
            let mounts = read_mounts();
            for (dev, mnt, fs) in mounts.iter().rev() {
                if mnt == b"/" || mnt == b"/proc" || mnt == b"/sys" || mnt == b"/dev" {
                    continue;
                }
                if !type_ok(fs) {
                    continue;
                }

                if umount_one(mnt, flags, retry_ro) != 0 {
                    rc |= umount_one(dev, flags, false);
                } else {
                    let _ = rc;
                }
            }
            return Ok(rc);
        }
        if pos.is_empty() {
            eprintln!("umount: no target");
            return Ok(1);
        }
        let mut rc = 0;
        for t in pos {
            rc |= umount_one(t, flags, retry_ro);
        }
        Ok(rc)
    }
}

pub struct DmesgApplet;
impl Applet for DmesgApplet {
    fn name(&self) -> &'static str {
        "dmesg"
    }
    fn description(&self) -> &'static str {
        "Print the kernel ring buffer"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut raw = false;
        let mut size: usize = usize::MAX;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
            } else if b == b"-r" {
                raw = true;
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("dmesg: option requires an argument -- 'n'");
                    return Ok(1);
                }
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("dmesg: option requires an argument -- 's'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                {
                    Some(n) => size = n,
                    None => {
                        eprintln!("dmesg: invalid size");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("dmesg: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        let path_c = match CString::new("/dev/kmsg") {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe {
            libc::open(
                path_c.as_ptr(),
                libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            eprintln!(
                "dmesg: cannot open /dev/kmsg: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut f = unsafe { fs::File::from_raw_fd(fd) };
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        let mut total = 0usize;
        let mut tail: Vec<u8> = Vec::new();
        loop {
            if total >= size {
                break;
            }
            let cap = buf.len().min(size - total);
            match f.read(&mut buf[..cap]) {
                Ok(0) => break,
                Ok(n) => {
                    total += n;
                    let mut start = 0;
                    for (k, &c) in buf[..n].iter().enumerate() {
                        if c == b'\n' {
                            tail.extend_from_slice(&buf[start..k]);
                            emit_kmsg(&mut out, &tail, raw)?;
                            tail.clear();
                            start = k + 1;
                        }
                    }
                    tail.extend_from_slice(&buf[start..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    eprintln!("dmesg: read error: {}", e);
                    return Ok(1);
                }
            }
        }
        if !tail.is_empty() {
            emit_kmsg(&mut out, &tail, raw)?;
        }
        out.flush()?;

        Ok(0)
    }
}

fn emit_kmsg(out: &mut impl Write, rec: &[u8], raw: bool) -> io::Result<()> {
    if raw || rec.is_empty() {
        out.write_all(rec)?;
        out.write_all(b"\n")?;
        return Ok(());
    }

    match rec.iter().position(|&b| b == b';') {
        Some(p) => {
            out.write_all(&rec[p + 1..])?;
            out.write_all(b"\n")?;
        }
        None => {
            let mut msg = rec;
            if msg.starts_with(b"<") {
                if let Some(end) = msg.iter().position(|&b| b == b'>') {
                    msg = &msg[end + 1..];
                }
            }
            out.write_all(msg)?;
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

struct BlkDev {
    name: Vec<u8>,
    maj: u32,
    min: u32,
    size_bytes: u64,
    rm: u8,
    ro: u8,
    is_part: bool,
    mountpoint: Vec<u8>,
}

fn read_u64_file(p: &Path) -> u64 {
    fs::read(p)
        .ok()
        .and_then(|d| {
            let s = std::str::from_utf8(&d).ok()?.trim();
            s.parse::<u64>().ok()
        })
        .unwrap_or(0)
}

fn parse_majmin(b: &[u8]) -> Option<(u32, u32)> {
    let s = std::str::from_utf8(b).ok()?.trim();
    let (a, c) = s.split_once(':')?;
    Some((a.trim().parse().ok()?, c.trim().parse().ok()?))
}

pub struct LsblkApplet;
impl Applet for LsblkApplet {
    fn name(&self) -> &'static str {
        "lsblk"
    }
    fn description(&self) -> &'static str {
        "List block devices"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut bytes = false;
        let mut nodeps = false;
        let mut exclude: Option<Vec<u32>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" || b == b"--all" {
                show_all = true;
            } else if b == b"-b" || b == b"--bytes" {
                bytes = true;
            } else if b == b"-d" || b == b"--nodeps" {
                nodeps = true;
            } else if b == b"-e" || b == b"--exclude" {
                i += 1;
                if i >= args.len() {
                    eprintln!("lsblk: option requires an argument");
                    return Ok(1);
                }
                exclude = Some(parse_maj_list(args[i].as_bytes()));
            } else if b.starts_with(b"-e") && b.len() > 2 {
                exclude = Some(parse_maj_list(&b[2..]));
            } else {
                eprintln!("lsblk: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }

        let excluded: Vec<u32> = if show_all {
            Vec::new()
        } else {
            exclude.unwrap_or_else(|| vec![1])
        };

        let mut mounted: Vec<(Vec<u8>, u64)> = Vec::new();
        for line in fs::read("/proc/mounts")
            .unwrap_or_default()
            .split(|&b| b == b'\n')
        {
            let mut parts = line.split(|&b| b == b' ');
            parts.next();
            if let Some(mnt) = parts.next() {
                let mp = Path::new(std::ffi::OsStr::from_bytes(mnt));
                if let Ok(st) = fs::metadata(mp) {
                    use std::os::unix::fs::MetadataExt;
                    mounted.push((mnt.to_vec(), st.dev()));
                }
            }
        }
        let mut devs: Vec<BlkDev> = Vec::new();
        let mut names: Vec<Vec<u8>> = fs::read_dir("/sys/block")
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().as_bytes().to_vec())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        for name in &names {
            let base = Path::new("/sys/block").join(std::ffi::OsStr::from_bytes(name));
            let dev_raw = fs::read(base.join("dev")).unwrap_or_default();
            let (maj, min) = match parse_majmin(&dev_raw) {
                Some(v) => v,
                None => continue,
            };
            if excluded.contains(&maj) {
                continue;
            }
            let want = libc::makedev(maj, min);
            let mp = mounted
                .iter()
                .find(|(_, d)| *d == want)
                .map(|(m, _)| m.clone())
                .unwrap_or_default();
            devs.push(BlkDev {
                name: name.clone(),
                maj,
                min,
                size_bytes: read_u64_file(&base.join("size")).saturating_mul(512),
                rm: read_u64_file(&base.join("removable")) as u8,
                ro: read_u64_file(&base.join("ro")) as u8,
                is_part: false,
                mountpoint: mp,
            });
            if nodeps {
                continue;
            }
            let mut kids: Vec<Vec<u8>> = fs::read_dir(&base)
                .map(|d| {
                    d.flatten()
                        .map(|e| e.file_name().as_bytes().to_vec())
                        .collect()
                })
                .unwrap_or_default();
            kids.sort();
            for kid in &kids {
                let kp = base.join(std::ffi::OsStr::from_bytes(kid));
                if !kp.join("partition").exists() {
                    continue;
                }
                let kdev = fs::read(kp.join("dev")).unwrap_or_default();
                let (kmaj, kmin) = match parse_majmin(&kdev) {
                    Some(v) => v,
                    None => continue,
                };

                let kwant = libc::makedev(kmaj, kmin);
                let kmp = mounted
                    .iter()
                    .find(|(_, d)| *d == kwant)
                    .map(|(m, _)| m.clone())
                    .unwrap_or_default();
                devs.push(BlkDev {
                    name: kid.clone(),
                    maj: kmaj,
                    min: kmin,
                    size_bytes: read_u64_file(&kp.join("size")).saturating_mul(512),
                    rm: read_u64_file(&kp.join("removable")) as u8,
                    ro: read_u64_file(&kp.join("ro")) as u8,
                    is_part: true,
                    mountpoint: kmp,
                });
            }
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        out.write_all(b"NAME MAJ:MIN RM SIZE RO TYPE MOUNTPOINT\n")?;

        let mut line: Vec<u8> = Vec::new();
        for d in &devs {
            line.clear();
            if d.is_part {
                line.extend_from_slice(b"  ");
            }
            line.extend_from_slice(&d.name);
            line.push(b' ');
            put_num(&mut line, d.maj as u64);
            line.push(b':');
            put_num(&mut line, d.min as u64);
            line.push(b' ');
            line.push(b'0' + (d.rm & 1));
            line.push(b' ');
            if bytes {
                put_num(&mut line, d.size_bytes);
            } else {
                put_human(&mut line, d.size_bytes);
            }
            line.push(b' ');
            line.push(b'0' + (d.ro & 1));
            line.push(b' ');

            let typ: &[u8] = if d.is_part {
                b"part"
            } else if d.name.starts_with(b"loop") {
                b"loop"
            } else if d.name.starts_with(b"ram") {
                b"ram"
            } else {
                b"disk"
            };
            line.extend_from_slice(typ);
            line.push(b' ');
            line.extend_from_slice(&d.mountpoint);
            line.push(b'\n');
            out.write_all(&line)?;
        }
        Ok(0)
    }
}

fn parse_maj_list(b: &[u8]) -> Vec<u32> {
    b.split(|&c| c == b',')
        .filter_map(|s| std::str::from_utf8(s).ok()?.trim().parse::<u32>().ok())
        .collect()
}

pub struct FlockApplet;
impl Applet for FlockApplet {
    fn name(&self) -> &'static str {
        "flock"
    }
    fn description(&self) -> &'static str {
        "Lock a file and run a command"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut shared = false;
        let mut nonblock = false;
        let mut unlock = false;
        let mut pos: Vec<&[u8]> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b's' => shared = true,
                        b'x' => shared = false,
                        b'n' => nonblock = true,
                        b'u' => unlock = true,
                        _ => {
                            eprintln!("flock: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(b);
            }
        }
        if pos.is_empty() {
            eprintln!("flock: no file specified");
            return Ok(1);
        }
        let mut op: libc::c_int = if unlock {
            libc::LOCK_UN
        } else if shared {
            libc::LOCK_SH
        } else {
            libc::LOCK_EX
        };
        if nonblock {
            op |= libc::LOCK_NB;
        }
        let target = pos[0];
        let cmd_args = &pos[1..];

        if !target.is_empty() && target.iter().all(|c| c.is_ascii_digit()) {
            let fd: i32 = std::str::from_utf8(target)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);
            if fd < 0 {
                eprintln!("flock: bad file descriptor");
                return Ok(1);
            }

            if unsafe { libc::flock(fd, op) } != 0 {
                eprintln!(
                    "flock: {}: {}",
                    String::from_utf8_lossy(target),
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
            if cmd_args.is_empty() {
                return Ok(0);
            }
            return run_cmd(cmd_args);
        }
        let path = Path::new(std::ffi::OsStr::from_bytes(target));
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path);
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                eprintln!("flock: {}: {}", String::from_utf8_lossy(target), e);
                return Ok(1);
            }
        };
        use std::os::unix::io::AsRawFd;

        if unsafe { libc::flock(file.as_raw_fd(), op) } != 0 {
            eprintln!(
                "flock: {}: {}",
                String::from_utf8_lossy(target),
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        if cmd_args.is_empty() {
            return Ok(0);
        }
        let rc = run_cmd(cmd_args);
        drop(file);
        rc
    }
}

fn run_cmd(cmd_args: &[&[u8]]) -> Result<i32> {
    let prog = String::from_utf8_lossy(cmd_args[0]).into_owned();
    let rest: Vec<String> = cmd_args[1..]
        .iter()
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    match Command::new(&prog).args(&rest).status() {
        Ok(st) => Ok(st.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("flock: {}: {}", prog, e);
            Ok(127)
        }
    }
}

struct Tok<'a> {
    v: &'a [&'a [u8]],
    i: usize,
}

impl<'a> Tok<'a> {
    fn peek(&self) -> Option<&'a [u8]> {
        self.v.get(self.i).copied()
    }
    fn next(&mut self) -> Option<&'a [u8]> {
        let t = self.v.get(self.i).copied();
        if t.is_some() {
            self.i += 1;
        }
        t
    }
}

fn eval_or(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    let mut v = eval_and(t)?;
    while t.peek() == Some(b"-o") {
        t.next();
        let rhs = eval_and(t)?;
        v = v || rhs;
    }
    Ok(v)
}

fn eval_and(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    let mut v = eval_not(t)?;
    while t.peek() == Some(b"-a") {
        t.next();
        let rhs = eval_not(t)?;
        v = v && rhs;
    }
    Ok(v)
}

fn eval_not(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    if t.peek() == Some(b"!") {
        t.next();
        Ok(!eval_not(t)?)
    } else {
        eval_primary(t)
    }
}

fn parse_int(s: &[u8]) -> Option<i64> {
    let s = std::str::from_utf8(s).ok()?;
    if s.is_empty() {
        return None;
    }
    let (neg, digs) = match s.as_bytes()[0] {
        b'-' => (true, &s[1..]),
        b'+' => (false, &s[1..]),
        _ => (false, s),
    };
    if digs.is_empty() || !digs.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut v: i64 = 0;
    for c in digs.bytes() {
        v = v.checked_mul(10)?.checked_add((c - b'0') as i64)?;
    }
    Some(if neg { -v } else { v })
}

fn file_unary(op: &[u8], path: &[u8]) -> std::result::Result<bool, Vec<u8>> {
    let p = Path::new(std::ffi::OsStr::from_bytes(path));
    let meta_follow = || fs::metadata(p);
    let meta_link = || fs::symlink_metadata(p);
    match op {
        b"-e" => Ok(meta_follow().is_ok()),
        b"-f" => Ok(meta_follow().map(|m| m.is_file()).unwrap_or(false)),
        b"-d" => Ok(meta_follow().map(|m| m.is_dir()).unwrap_or(false)),
        b"-s" => Ok(meta_follow().map(|m| m.len() > 0).unwrap_or(false)),
        b"-L" | b"-h" => Ok(meta_link()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)),
        b"-c" => Ok(meta_follow()
            .map(|m| m.file_type().is_char_device())
            .unwrap_or(false)),
        b"-b" => Ok(meta_follow()
            .map(|m| m.file_type().is_block_device())
            .unwrap_or(false)),
        b"-p" => Ok(meta_follow()
            .map(|m| m.file_type().is_fifo())
            .unwrap_or(false)),
        b"-S" => Ok(meta_follow()
            .map(|m| m.file_type().is_socket())
            .unwrap_or(false)),
        b"-r" | b"-w" | b"-x" => {
            let c = match CString::new(path) {
                Ok(c) => c,
                Err(_) => return Ok(false),
            };
            let mode = if op == b"-r" {
                libc::R_OK
            } else if op == b"-w" {
                libc::W_OK
            } else {
                libc::X_OK
            };

            Ok(unsafe { libc::access(c.as_ptr(), mode) } == 0)
        }
        b"-t" => {
            let fd: libc::c_int = std::str::from_utf8(path)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);
            if fd < 0 {
                return Ok(false);
            }

            Ok(unsafe { libc::isatty(fd) } == 1)
        }
        _ => Err(b"unknown unary operator".to_vec()),
    }
}

fn is_unary(op: &[u8]) -> bool {
    matches!(
        op,
        b"-e"
            | b"-f"
            | b"-d"
            | b"-r"
            | b"-w"
            | b"-x"
            | b"-s"
            | b"-L"
            | b"-h"
            | b"-c"
            | b"-b"
            | b"-p"
            | b"-S"
            | b"-n"
            | b"-z"
            | b"-t"
    )
}

fn is_binop(op: &[u8]) -> bool {
    matches!(
        op,
        b"=" | b"=="
            | b"!="
            | b"-eq"
            | b"-ne"
            | b"-gt"
            | b"-ge"
            | b"-lt"
            | b"-le"
            | b"-nt"
            | b"-ot"
            | b"-ef"
    )
}

fn eval_binary(a: &[u8], op: &[u8], c: &[u8]) -> std::result::Result<bool, Vec<u8>> {
    match op {
        b"=" | b"==" => Ok(a == c),
        b"!=" => Ok(a != c),
        b"-eq" | b"-ne" | b"-gt" | b"-ge" | b"-lt" | b"-le" => {
            let x = parse_int(a).ok_or_else(|| b"invalid integer".to_vec())?;
            let y = parse_int(c).ok_or_else(|| b"invalid integer".to_vec())?;
            Ok(match op {
                b"-eq" => x == y,
                b"-ne" => x != y,
                b"-gt" => x > y,
                b"-ge" => x >= y,
                b"-lt" => x < y,
                _ => x <= y,
            })
        }
        b"-nt" | b"-ot" | b"-ef" => {
            let pa = Path::new(std::ffi::OsStr::from_bytes(a));
            let pc = Path::new(std::ffi::OsStr::from_bytes(c));
            let ma = fs::metadata(pa).ok();
            let mc = fs::metadata(pc).ok();
            Ok(match op {
                b"-ef" => match (ma, mc) {
                    (Some(x), Some(y)) => {
                        use std::os::unix::fs::MetadataExt;
                        x.dev() == y.dev() && x.ino() == y.ino()
                    }
                    _ => false,
                },
                _ => {
                    let ta = ma.and_then(|m| m.modified().ok());
                    let tc = mc.and_then(|m| m.modified().ok());
                    match (ta, tc) {
                        (Some(x), Some(y)) => {
                            if op == b"-nt" {
                                x > y
                            } else {
                                x < y
                            }
                        }
                        _ => false,
                    }
                }
            })
        }
        _ => Err(b"unknown binary operator".to_vec()),
    }
}

fn eval_primary(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    match t.next() {
        None => Err(b"missing argument".to_vec()),
        Some(b"(") => {
            let v = eval_or(t)?;
            match t.next() {
                Some(b")") => Ok(v),
                _ => Err(b"missing ')'".to_vec()),
            }
        }
        Some(a) => {
            if is_unary(a) {
                let arg = t.next().ok_or_else(|| b"argument expected".to_vec())?;
                if a == b"-n" {
                    return Ok(!arg.is_empty());
                }
                if a == b"-z" {
                    return Ok(arg.is_empty());
                }
                return file_unary(a, arg);
            }

            if let Some(op) = t.peek() {
                if is_binop(op) {
                    t.next();
                    let c = t.next().ok_or_else(|| b"argument expected".to_vec())?;
                    return eval_binary(a, op, c);
                }
            }

            if is_unary(a)
                || is_binop(a)
                || a == b"!"
                || a == b"("
                || a == b")"
                || a == b"-a"
                || a == b"-o"
            {
                return Err(b"argument expected".to_vec());
            }
            Ok(!a.is_empty())
        }
    }
}

fn run_test(name: &str, args: &[OsString]) -> Result<i32> {
    let raw: Vec<&[u8]> = args.iter().map(|a| a.as_bytes()).collect();
    let expr: &[&[u8]] = if name == "[" {
        if raw.is_empty() || raw[raw.len() - 1] != b"]" {
            eprintln!("[: missing ']'");
            return Ok(2);
        }
        &raw[..raw.len() - 1]
    } else {
        &raw
    };
    if expr.is_empty() {
        return Ok(1);
    }
    let mut t = Tok { v: expr, i: 0 };
    match eval_or(&mut t) {
        Ok(v) => {
            if t.i != expr.len() {
                eprintln!("{}: too many arguments", name);
                return Ok(2);
            }
            Ok(if v { 0 } else { 1 })
        }
        Err(msg) => {
            eprintln!("{}: {}", name, String::from_utf8_lossy(&msg));
            Ok(2)
        }
    }
}

pub struct TestApplet;
impl Applet for TestApplet {
    fn name(&self) -> &'static str {
        "test"
    }
    fn description(&self) -> &'static str {
        "Evaluate a conditional expression"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_test("test", args)
    }
}

pub struct LBracketApplet;
impl Applet for LBracketApplet {
    fn name(&self) -> &'static str {
        "["
    }
    fn description(&self) -> &'static str {
        "Evaluate a conditional expression (alias for test)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_test("[", args)
    }
}
