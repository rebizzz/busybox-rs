use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self};
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
