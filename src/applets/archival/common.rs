use crate::applets::archival::inflate::inflate_all_consumed;
use crate::core::fs::open_or_stdin;
use crate::core::Result;
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
pub use std::mem::size_of;

pub const fn ior(magic: u8, nr: u8, size: usize) -> u64 {
    (2u64 << 30) | ((magic as u64) << 8) | (nr as u64) | ((size as u64) << 16)
}

pub const fn iow(magic: u8, nr: u8, size: usize) -> u64 {
    (1u64 << 30) | ((magic as u64) << 8) | (nr as u64) | ((size as u64) << 16)
}
pub fn write_octal(buf: &mut [u8], off: usize, width: usize, val: u64) {
    let s = format!("{:0>1$o}", val, width - 1);
    let b = s.as_bytes();
    let n = b.len().min(width - 1);
    buf[off..off + n].copy_from_slice(&b[b.len() - n..]);
    buf[off + width - 1] = 0;
}

pub fn parse_octal(b: &[u8]) -> u64 {
    let mut v = 0u64;
    for &c in b {
        if c == 0 || c == b' ' {
            if v != 0 {
                break;
            }
            continue;
        }
        if !(b'0'..=b'7').contains(&c) {
            break;
        }
        v = v * 8 + (c - b'0') as u64;
    }
    v
}

pub fn parse_hex8(b: &[u8]) -> u64 {
    let s = std::str::from_utf8(b).unwrap_or("");
    u64::from_str_radix(s.trim_matches('\0').trim(), 16).unwrap_or(0)
}

pub fn copy_stream<R: Read + ?Sized, W: Write + ?Sized>(r: &mut R, w: &mut W) -> std::io::Result<u64> {
    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        w.write_all(&buf[..n])?;
        total += n as u64;
    }
    Ok(total)
}

pub fn build_crc_table() -> [u32; 256] {
    let mut tab = [0u32; 256];
    for i in 0..256 {
        let mut c = (i as u32) << 24;
        for _ in 0..8 {
            c = if c & 0x8000_0000 != 0 {
                (c << 1) ^ 0x04C1_1DB7
            } else {
                c << 1
            };
        }
        tab[i as usize] = c;
    }
    tab
}

pub fn cksum_update(tab: &[u32; 256], mut crc: u32, data: &[u8]) -> u32 {
    for &b in data {
        crc = tab[((crc >> 24) as u8 ^ b) as usize] ^ (crc << 8);
    }
    crc
}

pub fn gzip_copy(in_path: &Path, out: &mut dyn Write) -> Result<(u64, bool)> {
    let mut inp = open_or_stdin(in_path)?;
    let mut magic = [0u8; 2];
    let mut n0 = 0;
    while n0 < 2 {
        match inp.read(&mut magic[n0..2])? {
            0 => break,
            n => n0 += n,
        }
    }
    let is_gz = n0 == 2 && magic == [0x1f, 0x8b];
    if n0 > 0 {
        out.write_all(&magic[..n0])?;
    }
    let n = copy_stream(&mut inp, out)?;
    Ok((n + n0 as u64, is_gz))
}

pub fn run_unpacker(args: &[OsString], name: &str, ext: &str) -> Result<i32> {
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

pub fn ab(a: &OsString) -> &[u8] {
    a.as_bytes()
}

pub fn u16le(d: &[u8], o: usize) -> usize {
    d[o] as usize | ((d[o + 1] as usize) << 8)
}

pub fn u32le(d: &[u8], o: usize) -> u32 {
    d[o] as u32 | ((d[o + 1] as u32) << 8) | ((d[o + 2] as u32) << 16) | ((d[o + 3] as u32) << 24)
}

pub fn u32be(d: &[u8], o: usize) -> u32 {
    ((d[o] as u32) << 24) | ((d[o + 1] as u32) << 16) | ((d[o + 2] as u32) << 8) | (d[o + 3] as u32)
}

pub fn read_all_bytes(p: Option<&Path>) -> std::io::Result<Vec<u8>> {
    let mut v = Vec::new();
    match p {
        Some(path) => File::open(path)?.read_to_end(&mut v)?,
        None => std::io::stdin().read_to_end(&mut v)?,
    };
    Ok(v)
}

pub fn stdout_write(data: &[u8]) -> std::io::Result<()> {
    std::io::stdout().lock().write_all(data)
}

pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut tab = [0u32; 256];
    for (i, slot) in tab.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = tab[((crc ^ (b as u32)) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

pub fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 10 && data[0] == 0x1F && data[1] == 0x8B && data[2] == 8
}

pub fn decode_gzip_members(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut cur = 0;
    let mut out = Vec::new();
    while cur < data.len() {
        if cur + 10 > data.len() || data[cur] != 0x1F || data[cur + 1] != 0x8B || data[cur + 2] != 8 {
            if cur == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not gzip"));
            }
            break;
        }
        let flags = data[cur + 3];
        let mut h = cur + 10;
        if flags & 4 != 0 {
            if h + 2 > data.len() {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "bad extra len"));
            }
            let xlen = u16le(data, h);
            h += 2 + xlen;
        }
        if flags & 8 != 0 {
            while h < data.len() && data[h] != 0 {
                h += 1;
            }
            h += 1;
        }
        if flags & 16 != 0 {
            while h < data.len() && data[h] != 0 {
                h += 1;
            }
            h += 1;
        }
        if flags & 2 != 0 {
            h += 2;
        }
        if h > data.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "bad gzip header"));
        }
        let (decomp, consumed) = inflate_all_consumed(&data[h..])?;
        out.extend_from_slice(&decomp);
        cur = h + consumed;
        if cur + 8 > data.len() {
            break;
        }
        cur += 8;
    }
    Ok(out)
}

#[repr(C)]
pub struct MtdInfoUser {
    pub typ: u8,
    pub _pad: [u8; 3],
    pub flags: u32,
    pub size: u32,
    pub erasesize: u32,
    pub writesize: u32,
    pub oobsize: u32,
    pub _pad2: u64,
}

#[repr(C)]
pub struct EraseInfoUser {
    pub start: u32,
    pub length: u32,
}

#[repr(C)]
pub struct Mtop {
    pub op: i16,
    pub count: i32,
}

#[repr(C)]
pub struct UbiAttachReq {
    pub ubi_num: i32,
    pub mtd_num: i32,
    pub vid_hdr_offset: i32,
    pub padding: [u8; 12],
}

pub fn ioctl_err(dev: &str, e: std::io::Error) -> i32 {
    eprintln!("{dev}: ioctl: {e} (no such device or no hardware?)");
    1
}

pub fn open_dev(path: &str) -> std::io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

pub fn ubi_ctrl_open(given: Option<&str>) -> std::io::Result<(File, String)> {
    if let Some(p) = given {
        let f = OpenOptions::new().read(true).write(true).open(p)?;
        return Ok((f, p.to_string()));
    }
    for cand in ["/dev/ubi_ctrl", "/dev/ubi/ctrl"] {
        if let Ok(f) = OpenOptions::new().read(true).write(true).open(cand) {
            return Ok((f, cand.to_string()));
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "cannot open ubi_ctrl",
    ))
}

pub fn ubi_vol_ioctl(dev: &str, nr: u8, payload: &[u8]) -> i32 {
    let req: libc::c_ulong = (3u64 << 30
        | (b'o' as u64) << 8
        | nr as u64
        | (payload.len() as u64) << 16) as libc::c_ulong;
    let path = format!("/dev/{dev}");
    let f = match open_dev(&path) {
        Ok(f) => f,
        Err(e) => return ioctl_err(&path, e),
    };
    let ret = unsafe { libc::ioctl(f.as_raw_fd(), req, payload.as_ptr()) };
    if ret < 0 {
        return ioctl_err(&path, std::io::Error::last_os_error());
    }
    0
}

pub fn pw_lookup(name: &str) -> Option<(u32, u32)> {
    let c = CString::new(name).ok()?;
    let p = unsafe { libc::getpwnam(c.as_ptr()) };
    if !p.is_null() {
        return Some(unsafe { ((*p).pw_uid, (*p).pw_gid) });
    }
    if let Ok(u) = name.parse::<u32>() {
        return Some((u, u));
    }
    None
}

pub fn gr_lookup(name: &str) -> Option<u32> {
    let c = CString::new(name).ok()?;
    let g = unsafe { libc::getgrnam(c.as_ptr()) };
    if !g.is_null() {
        return Some(unsafe { (*g).gr_gid });
    }
    name.parse::<u32>().ok()
}

pub fn split_user_group(s: &str) -> (String, Option<String>) {
    if let Some((u, g)) = s.split_once(':') {
        (u.to_string(), Some(g.to_string()))
    } else if let Some((u, g)) = s.split_once('.') {
        (u.to_string(), Some(g.to_string()))
    } else {
        (s.to_string(), None)
    }
}

pub fn exec_prog(prog: &OsString, args: &[OsString], uid: Option<u32>, gid: Option<u32>) -> i32 {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    if let Some(g) = gid {
        cmd.gid(g);
    }
    if let Some(u) = uid {
        cmd.uid(u);
    }
    match cmd.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("{}: {}", prog.to_string_lossy(), e);
            111
        }
    }
}

pub fn apply_rlimit(res: u32, n: u64) -> std::io::Result<()> {
    let r = libc::rlimit {
        rlim_cur: n as libc::rlim_t,
        rlim_max: n as libc::rlim_t,
    };
    if unsafe { libc::setrlimit(res as _, &r) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn supervise_pid(dir: &Path) -> Option<i32> {
    let p = dir.join("supervise/pid");
    std::fs::read_to_string(p).ok()?.trim().parse().ok()
}

pub fn supervise_stat(dir: &Path) -> Option<String> {
    let p = dir.join("supervise/stat");
    std::fs::read_to_string(p).ok().map(|s| s.trim().to_string())
}

pub fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

pub fn sig_send(pid: i32, sig: i32) -> bool {
    unsafe { libc::kill(pid, sig) == 0 }
}
