use crate::core::Result;
use std::ffi::{CString, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub fn print_bytes(bytes: &[u8]) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(bytes);
}

pub fn put_num(buf: &mut Vec<u8>, mut n: u64) {
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

pub fn put_num_pad(buf: &mut Vec<u8>, n: u64, width: usize) {
    let start = buf.len();
    put_num(buf, n);
    let len = buf.len() - start;
    if len < width {
        let pad = width - len;
        let digits = buf[start..].to_vec();
        buf.truncate(start);
        for _ in 0..pad {
            buf.push(b' ');
        }
        buf.extend_from_slice(&digits);
    }
}

pub fn push_u64(out: &mut Vec<u8>, mut v: u64) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    let mut tmp = [0u8; 20];
    let mut n = 0;
    while v > 0 {
        tmp[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        out.push(tmp[n]);
    }
}

pub fn read_small(path: &str, buf: &mut [u8]) -> usize {
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut n = 0;
    while n < buf.len() {
        match f.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(r) => n += r,
            Err(_) => break,
        }
    }
    n
}

pub fn exec_prog(prog: &[u8], args: &[OsString]) -> i32 {
    let argv0 = CString::new(prog).unwrap_or_else(|_| CString::new("sh").unwrap());
    let mut cs: Vec<CString> = Vec::with_capacity(args.len() + 1);
    cs.push(argv0);
    for a in args {
        cs.push(CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()));
    }
    let mut ptrs: Vec<*const libc::c_char> = cs.iter().map(|c| c.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    unsafe {
        libc::execvp(ptrs[0], ptrs.as_ptr());
    }
    eprintln!(
        "{}: {}",
        String::from_utf8_lossy(prog),
        std::io::Error::last_os_error()
    );
    127
}

pub fn parse_u64_suffix(b: &[u8]) -> Option<u64> {
    if b.is_empty() {
        return None;
    }
    let (num, mul) = match b.last() {
        Some(c) if c.is_ascii_alphabetic() => (
            &b[..b.len() - 1],
            match c.to_ascii_uppercase() {
                b'K' => 1024u64,
                b'M' => 1024 * 1024,
                b'G' => 1024 * 1024 * 1024,
                b'T' => 1024 * 1024 * 1024 * 1024,
                _ => return None,
            },
        ),
        _ => (b, 1u64),
    };
    if num.is_empty() || !num.iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut v: u64 = 0;
    for &c in num {
        v = v.checked_mul(10)?.checked_add((c - b'0') as u64)?;
    }
    v.checked_mul(mul)
}

pub const PER_LINUX: libc::c_ulong = 0x0000;
pub const PER_LINUX32: libc::c_ulong = 0x0008;

pub fn do_personality(persona: libc::c_ulong, prog: Option<Vec<OsString>>) -> i32 {
    let cur = unsafe { libc::personality(0xffffffff) };
    if cur < 0 {
        eprintln!("personality: {}", std::io::Error::last_os_error());
        return 1;
    }
    if unsafe { libc::personality(persona) } < 0 {
        eprintln!("personality: {}", std::io::Error::last_os_error());
        return 1;
    }
    match prog {
        Some(cmd) if !cmd.is_empty() => exec_prog(cmd[0].as_bytes(), &cmd[1..]),
        _ => {
            let sh = b"/bin/sh";
            exec_prog(sh, &[])
        }
    }
}

pub fn chdir_cstr(p: &[u8]) -> bool {
    if let Ok(c) = CString::new(p) {
        unsafe { libc::chdir(c.as_ptr()) == 0 }
    } else {
        false
    }
}

pub const RTC_RD_TIME: libc::c_ulong = 0x8024_7009;

pub fn make_fat_fs(args: &[OsString]) -> Result<i32> {
    let mut fat_bits: u32 = 0;
    let mut label = [b' '; 11];
    let mut target: Option<&Path> = None;

    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"-F" && i + 1 < args.len() {
            i += 1;
            let s = args[i].to_string_lossy();
            fat_bits = s.parse().unwrap_or(0);
        } else if b == b"-n" && i + 1 < args.len() {
            i += 1;
            let s = args[i].as_bytes();
            let len = s.len().min(11);
            label[..len].copy_from_slice(&s[..len]);
        } else if !b.starts_with(b"-") && target.is_none() {
            target = Some(Path::new(&args[i]));
        }
        i += 1;
    }

    let target = match target {
        Some(t) => t,
        None => {
            eprintln!("mkfs.vfat: device required");
            return Ok(1);
        }
    };

    let mut f = match OpenOptions::new().read(true).write(true).open(target) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("mkfs.vfat: {}: {}", target.display(), e);
            return Ok(1);
        }
    };

    let dev_size = f.metadata().map(|m| m.len()).unwrap_or(0);
    let total_sectors = if dev_size > 0 { dev_size / 512 } else { 2880 };

    if fat_bits == 0 {
        if total_sectors < 8400 {
            fat_bits = 12;
        } else if total_sectors < 66600 {
            fat_bits = 16;
        } else {
            fat_bits = 32;
        }
    }

    let mut boot = [0u8; 512];

    boot[0] = 0xEB;
    boot[1] = 0x3C;
    boot[2] = 0x90;
    boot[3..11].copy_from_slice(b"MSDOS5.0");

    boot[11..13].copy_from_slice(&512u16.to_le_bytes());

    let spc: u8 = if total_sectors < 8400 {
        1
    } else if total_sectors < 66600 {
        4
    } else {
        8
    };
    boot[13] = spc;

    let reserved: u16 = if fat_bits == 32 { 32 } else { 1 };
    boot[14..16].copy_from_slice(&reserved.to_le_bytes());

    boot[16] = 2;

    let root_entries: u16 = if fat_bits == 32 { 0 } else { 512 };
    boot[17..19].copy_from_slice(&root_entries.to_le_bytes());

    if total_sectors < 65536 && fat_bits != 32 {
        boot[19..21].copy_from_slice(&(total_sectors as u16).to_le_bytes());
    } else {
        boot[32..36].copy_from_slice(&(total_sectors as u32).to_le_bytes());
    }

    boot[21] = 0xF8;

    if fat_bits == 32 {
        let fat_size = ((total_sectors / (spc as u64) * 4).div_ceil(512)) as u32;
        boot[36..40].copy_from_slice(&fat_size.to_le_bytes());
        boot[44..48].copy_from_slice(&2u32.to_le_bytes());
        boot[66] = 0x29;
        boot[71..82].copy_from_slice(&label);
        boot[82..90].copy_from_slice(b"FAT32   ");
    } else {
        let fat_size: u16 = if fat_bits == 12 {
            9
        } else {
            ((total_sectors / (spc as u64) * 2).div_ceil(512)) as u16
        };
        boot[22..24].copy_from_slice(&fat_size.to_le_bytes());
        boot[38] = 0x29;
        boot[43..54].copy_from_slice(&label);
        let ftype = if fat_bits == 12 {
            b"FAT12   "
        } else {
            b"FAT16   "
        };
        boot[54..62].copy_from_slice(ftype);
    }

    boot[510] = 0x55;
    boot[511] = 0xAA;

    if f.seek(SeekFrom::Start(0)).is_err() || f.write_all(&boot).is_err() {
        eprintln!("mkfs.vfat: failed to write boot sector");
        return Ok(1);
    }

    Ok(0)
}

pub fn make_ext2_fs(args: &[OsString]) -> Result<i32> {
    let mut block_size: u32 = 4096;
    let mut label = [0u8; 16];
    let mut target: Option<&Path> = None;

    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"-b" && i + 1 < args.len() {
            i += 1;
            block_size = args[i].to_string_lossy().parse().unwrap_or(4096);
        } else if b == b"-L" && i + 1 < args.len() {
            i += 1;
            let s = args[i].as_bytes();
            let len = s.len().min(16);
            label[..len].copy_from_slice(&s[..len]);
        } else if !b.starts_with(b"-") && target.is_none() {
            target = Some(Path::new(&args[i]));
        }
        i += 1;
    }

    let target = match target {
        Some(t) => t,
        None => {
            eprintln!("mkfs.ext2: device required");
            return Ok(1);
        }
    };

    let mut f = match OpenOptions::new().read(true).write(true).open(target) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("mkfs.ext2: {}: {}", target.display(), e);
            return Ok(1);
        }
    };

    let dev_size = f.metadata().map(|m| m.len()).unwrap_or(0);
    let total_blocks = if dev_size > 0 { dev_size / block_size as u64 } else { 1024 };

    let mut sb = [0u8; 1024];
    sb[0..4].copy_from_slice(&(total_blocks as u32 / 4).to_le_bytes());
    sb[4..8].copy_from_slice(&(total_blocks as u32).to_le_bytes());
    sb[12..16].copy_from_slice(&(total_blocks as u32 - 10).to_le_bytes());
    sb[16..20].copy_from_slice(&(total_blocks as u32 / 4 - 11).to_le_bytes());
    sb[24..28].copy_from_slice(&(block_size.trailing_zeros().saturating_sub(10)).to_le_bytes());
    sb[56..58].copy_from_slice(&0xEF53u16.to_le_bytes());
    sb[58..60].copy_from_slice(&1u16.to_le_bytes());
    sb[120..136].copy_from_slice(&label);

    if f.seek(SeekFrom::Start(1024)).is_err() || f.write_all(&sb).is_err() {
        eprintln!("mkfs.ext2: failed to write superblock");
        return Ok(1);
    }

    Ok(0)
}

pub const FIFREEZE: libc::c_ulong = 0xc0045877;
pub const FITHAW: libc::c_ulong = 0xc0045878;
pub const BLKRRPART: libc::c_ulong = 0x125f;
pub const VT_OPENQRY: libc::c_ulong = 0x5600;
pub const VT_ACTIVATE: libc::c_ulong = 0x5606;
pub const VT_WAITACTIVE: libc::c_ulong = 0x5607;
pub const TIOCGWINSZ: libc::c_ulong = 0x5413;
pub const TIOCLINUX: libc::c_ulong = 0x541c;
pub const TIOCGSERIAL: libc::c_ulong = 0x541e;
pub const TIOCSSERIAL: libc::c_ulong = 0x541f;
pub const NBD_SET_SOCK: libc::c_ulong = 0xab00;
pub const NBD_SET_BLKSIZE: libc::c_ulong = 0xab01;
pub const NBD_DO_IT: libc::c_ulong = 0xab03;
pub const NBD_CLEAR_SOCK: libc::c_ulong = 0xab04;
pub const NBD_CLEAR_QUE: libc::c_ulong = 0xab05;
pub const NBD_DISCONNECT: libc::c_ulong = 0xab08;
pub const NBD_SET_TIMEOUT: libc::c_ulong = 0xab09;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct SerialStruct {
    pub type_: libc::c_int,
    pub line: libc::c_int,
    pub port: libc::c_uint,
    pub irq: libc::c_int,
    pub flags: libc::c_int,
    pub xmit_fifo_size: libc::c_int,
    pub custom_divisor: libc::c_int,
    pub baud_base: libc::c_int,
    pub close_delay: libc::c_ushort,
    pub io_type: libc::c_char,
    pub reserved_char: [libc::c_char; 1],
    pub hub6: libc::c_int,
    pub closing_wait: libc::c_ushort,
    pub closing_wait2: libc::c_ushort,
    pub iomem_base: *mut libc::c_uchar,
    pub iomem_reg_shift: libc::c_ushort,
    pub port_high: libc::c_uint,
    pub iomap_base: libc::c_ulong,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Winsize {
    pub ws_row: libc::c_ushort,
    pub ws_col: libc::c_ushort,
    pub ws_xpixel: libc::c_ushort,
    pub ws_ypixel: libc::c_ushort,
}

pub fn path_cstr(p: &Path) -> Option<CString> {
    CString::new(p.as_os_str().as_bytes()).ok()
}

pub fn put_human(buf: &mut Vec<u8>, bytes: u64) {
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

pub fn read_mounts() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let data = fs::read("/proc/mounts").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(|&b| b == b' ');
        let dev = parts.next().unwrap_or(b"").to_vec();
        let mnt = parts.next().unwrap_or(b"").to_vec();
        let fst = parts.next().unwrap_or(b"").to_vec();
        out.push((dev, mnt, fst));
    }
    out
}

pub struct FstabEntry {
    pub dev: Vec<u8>,
    pub mnt: Vec<u8>,
    pub fst: Vec<u8>,
    pub opt: Vec<u8>,
}

pub fn read_fstab(path: &Path) -> Vec<FstabEntry> {
    let data = fs::read(path).unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        let trimmed = line.trim_ascii_start();
        if trimmed.is_empty() || trimmed.starts_with(b"#") {
            continue;
        }
        let parts: Vec<&[u8]> = trimmed.split(|&b| b == b' ' || b == b'\t').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 4 {
            out.push(FstabEntry {
                dev: parts[0].to_vec(),
                mnt: parts[1].to_vec(),
                fst: parts[2].to_vec(),
                opt: parts[3].to_vec(),
            });
        }
    }
    out
}

pub fn fold_opt(name: &[u8], flags: &mut libc::c_ulong, data: &mut Vec<Vec<u8>>) {
    match name {
        b"ro" => *flags |= libc::MS_RDONLY,
        b"rw" => *flags &= !libc::MS_RDONLY,
        b"nosuid" => *flags |= libc::MS_NOSUID,
        b"suid" => *flags &= !libc::MS_NOSUID,
        b"nodev" => *flags |= libc::MS_NODEV,
        b"dev" => *flags &= !libc::MS_NODEV,
        b"noexec" => *flags |= libc::MS_NOEXEC,
        b"exec" => *flags &= !libc::MS_NOEXEC,
        b"sync" => *flags |= libc::MS_SYNCHRONOUS,
        b"async" => *flags &= !libc::MS_SYNCHRONOUS,
        b"remount" => *flags |= libc::MS_REMOUNT,
        b"bind" => *flags |= libc::MS_BIND,
        b"rbind" => *flags |= libc::MS_BIND | libc::MS_REC,
        b"move" => *flags |= libc::MS_MOVE,
        b"defaults" => {}
        other => data.push(other.to_vec()),
    }
}

pub fn proc_filesystems() -> Vec<Vec<u8>> {
    let data = fs::read("/proc/filesystems").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        let trimmed = line.trim_ascii();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&[u8]> = trimmed.split(|&b| b == b'\t' || b == b' ').filter(|s| !s.is_empty()).collect();
        if parts.len() == 1 {
            out.push(parts[0].to_vec());
        } else if parts.len() >= 2 && parts[0] != b"nodev" {
            out.push(parts[1].to_vec());
        }
    }
    out
}

pub fn mount_one(
    dev: &[u8],
    dir: &[u8],
    fstype: Option<&[u8]>,
    flags: libc::c_ulong,
    opts: &[Vec<u8>],
) -> i32 {
    let dev_c = match CString::new(dev) {
        Ok(c) => c,
        Err(_) => return 1,
    };
    let dir_c = match CString::new(dir) {
        Ok(c) => c,
        Err(_) => return 1,
    };
    let data_bytes = if opts.is_empty() {
        Vec::new()
    } else {
        opts.join(&b","[..])
    };
    let data_c = CString::new(data_bytes).ok();
    let data_ptr = data_c
        .as_ref()
        .map(|c| c.as_ptr() as *const libc::c_void)
        .unwrap_or(std::ptr::null());

    let do_call = |fs: *const libc::c_char| -> i32 {
        let ret = unsafe {
            libc::mount(
                dev_c.as_ptr(),
                dir_c.as_ptr(),
                fs,
                flags,
                data_ptr,
            )
        };
        if ret == 0 {
            0
        } else {
            1
        }
    };

    if let Some(fs) = fstype {
        let fs_c = match CString::new(fs) {
            Ok(c) => c,
            Err(_) => return 1,
        };
        let r = do_call(fs_c.as_ptr());
        if r != 0 {
            let err = io::Error::last_os_error();
            eprintln!(
                "mount: mounting {} on {} failed: {}",
                String::from_utf8_lossy(dev),
                String::from_utf8_lossy(dir),
                err
            );
        }
        r
    } else if (flags & (libc::MS_BIND | libc::MS_MOVE)) != 0 {
        let r = do_call(std::ptr::null());
        if r != 0 {
            let err = io::Error::last_os_error();
            eprintln!(
                "mount: mounting {} on {} failed: {}",
                String::from_utf8_lossy(dev),
                String::from_utf8_lossy(dir),
                err
            );
        }
        r
    } else {
        let candidates = proc_filesystems();
        let fallback = [b"ext4".as_slice(), b"ext3", b"ext2", b"vfat"];
        let all: Vec<&[u8]> = candidates
            .iter()
            .map(|c| c.as_slice())
            .chain(fallback.iter().copied())
            .collect();

        for candidate in all {
            if let Ok(fs_c) = CString::new(candidate) {
                if do_call(fs_c.as_ptr()) == 0 {
                    return 0;
                }
            }
        }
        let err = io::Error::last_os_error();
        eprintln!(
            "mount: mounting {} on {} failed: {}",
            String::from_utf8_lossy(dev),
            String::from_utf8_lossy(dir),
            err
        );
        1
    }
}
