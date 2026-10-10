use crate::core::Result;
use crate::core::digest::{Digest, Md5, Sha1, Sha256, Sha512};
use crate::core::fs::open_or_stdin;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[macro_export]
macro_rules! bb_applet {
    ($t:ident, $n:literal, $d:literal, $f:ident) => {
        pub struct $t;
        impl $crate::core::Applet for $t {
            fn name(&self) -> &'static str {
                $n
            }
            fn description(&self) -> &'static str {
                $d
            }
            fn run(&self, args: &[std::ffi::OsString]) -> $crate::core::Result<i32> {
                $f(args)
            }
        }
    };
}

#[macro_export]
macro_rules! applet {
    ($($tokens:tt)*) => {
        $crate::bb_applet!($($tokens)*);
    };
}

#[allow(dead_code)]
pub fn ab(a: &OsString) -> &[u8] {
    a.as_os_str().as_bytes()
}

#[allow(dead_code)]
pub fn lossy(a: &OsString) -> String {
    a.as_os_str().to_string_lossy().into_owned()
}

#[allow(dead_code)]
pub fn read_all(p: &OsStr) -> std::io::Result<Vec<u8>> {
    if p.as_bytes() == b"-" {
        let mut v = Vec::new();
        std::io::stdin().lock().read_to_end(&mut v)?;
        Ok(v)
    } else {
        let mut v = Vec::new();
        File::open(Path::new(p))?.read_to_end(&mut v)?;
        Ok(v)
    }
}

#[allow(dead_code)]
pub fn hex_up(out: &mut Vec<u8>, b: &[u8]) {
    const H: &[u8; 16] = b"0123456789abcdef";
    out.reserve(b.len() * 2);
    for &x in b {
        out.push(H[(x >> 4) as usize]);
        out.push(H[(x & 15) as usize]);
    }
}

#[allow(dead_code)]
pub fn hex_str(b: &[u8]) -> String {
    let mut v = Vec::with_capacity(b.len() * 2);
    hex_up(&mut v, b);
    String::from_utf8(v).unwrap_or_default()
}

#[allow(dead_code)]
pub fn wlock() -> std::io::StdoutLock<'static> {
    std::io::stdout().lock()
}


// Shared coreutils helpers from crypto_attr
use std::os::unix::io::AsRawFd;
pub fn dump_lines(out: &mut impl Write, data: &[u8], base: u64, cols: usize, up: bool) {
    let mut off = base;
    let mut line = Vec::with_capacity(128);
    for c in data.chunks(cols.max(1)) {
        line.clear();
        let _ = write!(line, "{:08x}  ", off);
        for &b in c {
            if up {
                let _ = write!(line, "{:02X} ", b);
            } else {
                line.extend_from_slice(&[
                    b"0123456789abcdef"[(b >> 4) as usize],
                    b"0123456789abcdef"[(b & 15) as usize],
                    b' ',
                ]);
            }
        }

        let _ = write!(line, " |");
        for &b in c {
            line.push(if (32..127).contains(&b) { b } else { b'.' });
        }
        line.push(b'|');
        line.push(b'\n');
        let _ = out.write_all(&line);
        off += c.len() as u64;
    }
}
pub fn b64_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = Vec::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16
            | (c.get(1).copied().unwrap_or(0) as u32) << 8
            | (c.get(2).copied().unwrap_or(0) as u32);
        o.push(T[(n >> 18) as usize & 63]);
        o.push(T[(n >> 12) as usize & 63]);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63]
        } else {
            b'='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63]
        } else {
            b'='
        });
    }
    o
}
pub fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => Some(64),
        _ => None,
    }
}
pub fn b64_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        match b64_val(c) {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(4) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::with_capacity(digs.len() / 4 * 3);
    for c in digs.chunks(4) {
        let pad = c.iter().rev().take_while(|&&v| v == 64).count();
        if pad > 2 {
            return Err("invalid input".to_string());
        }
        let mut n = 0u32;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 64 { 0 } else { v as u32 };
            n |= v << (18 - 6 * i);
        }
        o.push((n >> 16) as u8);
        if pad < 2 {
            o.push((n >> 8) as u8);
        }
        if pad < 1 {
            o.push(n as u8);
        }
    }
    Ok(o)
}
pub fn b32_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut o = Vec::with_capacity(d.len().div_ceil(5) * 8);
    for c in d.chunks(5) {
        let mut n = 0u64;
        for &b in c {
            n = (n << 8) | b as u64;
        }
        n <<= (5 - c.len()) * 8;
        let nd = (c.len() * 8).div_ceil(5);
        for i in 0..8 {
            o.push(if i < nd {
                T[(n >> (35 - 5 * i)) as usize & 31]
            } else {
                b'='
            });
        }
    }
    o
}
pub fn b32_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        let v = match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26 - 24 + 24 - 24),
            b'2'..=b'7' => Some(c - b'2' + 26),
            b'=' => Some(32),
            _ => None,
        };

        let v = match c {
            b'a'..=b'z' => Some(c - b'a'),
            _ => v,
        };
        match v {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(8) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::new();
    for c in digs.chunks(8) {
        let pad = c.iter().rev().take_while(|&&v| v == 32).count();
        let mut n = 0u64;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 32 { 0 } else { v as u64 };
            n |= v << (35 - 5 * i);
        }
        let nb = 5 - pad * 5 / 8;
        for i in 0..nb {
            o.push((n >> (32 - 8 * i)) as u8);
        }
    }
    Ok(o)
}
pub struct VolInfo {
    pub label: String,
    pub uuid: String,
    pub fstype: String,
}
pub fn probe_vol(path: &OsStr) -> Option<VolInfo> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(Path::new(path)).ok()?;

    let mut boot = [0u8; 512];
    if f.read_exact(&mut boot).is_ok() && boot[510] == 0x55 && boot[511] == 0xaa {
        let is_fat32 = boot[82..90] == *b"FAT32   ";
        let is_fat = is_fat32 || boot[54..62] == *b"FAT12   " || boot[54..62] == *b"FAT16   ";
        if is_fat {
            let (lab_off, ser_off) = if is_fat32 { (71, 67) } else { (43, 39) };
            let label = String::from_utf8_lossy(&boot[lab_off..lab_off + 11])
                .trim()
                .to_string();
            let ser = u32::from_le_bytes(boot[ser_off..ser_off + 4].try_into().unwrap_or([0; 4]));
            return Some(VolInfo {
                label,
                uuid: format!("{:04X}-{:04X}", ser >> 16, ser & 0xffff),
                fstype: "vfat".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut sb = [0u8; 256];
        if f.read_exact(&mut sb).is_ok() && sb[56] == 0x53 && sb[57] == 0xef {
            let label = String::from_utf8_lossy(&sb[120..136])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &sb[104..120];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "ext2/3/4".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut pg = vec![0u8; 4096];
        if f.read_exact(&mut pg).is_ok() && pg[4096 - 10..] == *b"SWAPSPACE2" {
            let label = String::from_utf8_lossy(&pg[1052..1068])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &pg[1036..1052];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "swap".to_string(),
            });
        }
    }
    None
}

pub fn candidate_devs() -> Vec<OsString> {
    let mut v: Vec<OsString> = Vec::new();
    if let Ok(m) = std::fs::read("/proc/mounts") {
        for line in m.split(|&c| c == b'\n') {
            if let Some(sp) = line.iter().position(|&c| c == b' ') {
                let src = &line[..sp];
                if src.starts_with(b"/dev/") {
                    v.push(OsString::from(OsStr::from_bytes(src)));
                }
            }
        }
    }
    if let Ok(sys) = std::fs::read_dir("/sys/block") {
        for e in sys.flatten() {
            let nm = e.file_name();
            let devp = format!("/dev/{}", nm.to_string_lossy());
            let o = OsString::from(&devp);
            if !v.contains(&o) {
                v.push(o);
            }
        }
    }
    for guess in [
        "/dev/sda1",
        "/dev/vda1",
        "/dev/nvme0n1p1",
        "/dev/mmcblk0p1",
        "/dev/dm-0",
    ] {
        let o = OsString::from(guess);
        if !v.contains(&o) {
            v.push(o);
        }
    }
    v
}
pub const F_GET: libc::c_ulong = 0x80086601;
pub const F_SET: libc::c_ulong = 0x40086602;
pub const ATTR_BITS: [(u8, libc::c_long); 13] = [
    (b's', 0x0000_0001),
    (b'u', 0x0000_0002),
    (b'c', 0x0000_0004),
    (b'S', 0x0000_0008),
    (b'i', 0x0000_0010),
    (b'a', 0x0000_0020),
    (b'A', 0x0000_0080),
    (b'd', 0x0000_0040),
    (b'D', 0x0001_0000),
    (b'E', 0x0000_0800),
    (b'e', 0x0008_0000),
    (b'I', 0x0000_1000),
    (b'j', 0x0000_4000),
];
pub const ATTR_ORDER: [u8; 13] = *b"sucSiaAdDEeIj";
pub fn bit_of(c: u8) -> Option<libc::c_long> {
    ATTR_BITS.iter().find(|&&(x, _)| x == c).map(|&(_, b)| b)
}
pub fn get_flags(p: &OsStr) -> std::io::Result<libc::c_long> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut fl: libc::c_long = 0;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_GET as _, &mut fl) };
    let _ = f;
    let _ = OsStr::from_bytes;
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(fl)
    }
}
pub fn set_flags(p: &OsStr, fl: libc::c_long) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut flm = fl;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_SET as _, &mut flm) };
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub fn flags_str(fl: libc::c_long) -> String {
    let mut s = String::with_capacity(20);
    for &c in &ATTR_ORDER {
        let b = bit_of(c).unwrap_or(0);
        s.push(if fl & b != 0 { c as char } else { '-' });
    }
    s
}
pub fn xenc(data: &[u8], fmt: &str) -> String {
    match fmt {
        "hex" => {
            let mut s = String::from("0x");
            for &b in data {
                use std::fmt::Write as _;
                let _ = write!(s, "{:02x}", b);
            }
            s
        }
        "base64" => String::from_utf8(b64_enc(data)).unwrap_or_default(),
        _ => String::from_utf8_lossy(data).into_owned(),
    }
}
pub fn ipc_key_rm(kind: u8, id_or_key: &str, by_key: bool) -> std::io::Result<()> {
    unsafe {
        if by_key {
            let key: i32 = if let Some(h) = id_or_key
                .strip_prefix("0x")
                .or_else(|| id_or_key.strip_prefix("0X"))
            {
                i32::from_str_radix(h, 16)
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad key"))?
            } else {
                id_or_key
                    .parse()
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad key"))?
            };
            let r = match kind {
                b'm' => {
                    let id = libc::shmget(key, 0, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut())
                }
                b's' => {
                    let id = libc::semget(key, 0, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::semctl(id, 0, libc::IPC_RMID)
                }
                _ => {
                    let id = libc::msgget(key, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut())
                }
            };
            if r < 0 {
                return Err(std::io::Error::last_os_error());
            }
        } else {
            let id: i32 = id_or_key
                .parse()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad id"))?;
            let r = match kind {
                b'm' => libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()),
                b's' => libc::semctl(id, 0, libc::IPC_RMID),
                _ => libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut()),
            };
            if r < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
pub fn read_stat() -> Vec<u8> {
    std::fs::read("/proc/stat").unwrap_or_default()
}
pub fn cpu_line(d: &[u8], prefix: &[u8]) -> Option<Vec<u64>> {
    for line in d.split(|&c| c == b'\n') {
        if line.starts_with(prefix) && line.get(prefix.len()) == Some(&b' ') {
            let mut v = Vec::new();
            for f in line[prefix.len() + 1..].split(|&c| c == b' ' || c == b'\t') {
                if f.is_empty() {
                    continue;
                }
                v.push(String::from_utf8_lossy(f).parse().unwrap_or(0));
            }
            return Some(v);
        }
    }
    None
}



#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl HashType {
    pub fn create_digest(&self) -> Box<dyn Digest> {
        match self {
            HashType::Md5 => Box::new(Md5::new()),
            HashType::Sha1 => Box::new(Sha1::new()),
            HashType::Sha256 => Box::new(Sha256::new()),
            HashType::Sha512 => Box::new(Sha512::new()),
        }
    }
}

pub fn hash_stream<R: Read>(digest: &mut dyn Digest, mut reader: R) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(())
}

pub fn hash_file_or_stdin(
    hash_type: HashType,
    applet_name: &str,
    file: &str,
) -> std::result::Result<String, ()> {
    let mut digest = hash_type.create_digest();
    if file == "-" {
        let stdin = io::stdin();
        let handle = stdin.lock();
        if let Err(e) = hash_stream(digest.as_mut(), handle) {
            eprintln!("{}: {}: {}", applet_name, file, e);
            return Err(());
        }
    } else {
        match File::open(Path::new(file)) {
            Ok(f) => {
                let reader = BufReader::new(f);
                if let Err(e) = hash_stream(digest.as_mut(), reader) {
                    eprintln!("{}: {}: {}", applet_name, file, e);
                    return Err(());
                }
            }
            Err(e) => {
                eprintln!("{}: can't open '{}': {}", applet_name, file, e);
                return Err(());
            }
        }
    }
    Ok(digest.finalize_hex())
}

pub fn run_hash_cmd(applet_name: &'static str, hash_type: HashType, args: &[OsString]) -> Result<i32> {
    let mut check_mode = false;
    let mut silent = false;
    let mut warn = false;
    let mut binary_flag = false;
    let mut files: Vec<String> = Vec::new();

    let mut parsing_opts = true;
    for arg in args {
        let s = arg.to_string_lossy();
        if parsing_opts && s == "--" {
            parsing_opts = false;
            continue;
        }
        if parsing_opts && s.starts_with('-') && s.len() > 1 && s != "-" {
            for ch in s[1..].chars() {
                match ch {
                    'c' => check_mode = true,
                    's' => silent = true,
                    'w' => warn = true,
                    'b' => binary_flag = true,
                    't' => binary_flag = false,
                    _ => {
                        eprintln!("{}: unrecognized option '{}'", applet_name, s);
                        return Ok(1);
                    }
                }
            }
            continue;
        }
        files.push(s.into_owned());
    }

    if (silent || warn) && !check_mode {
        eprintln!("{}: -s and -w require -c", applet_name);
        return Ok(1);
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    if check_mode {
        let mut overall_success = true;

        for file_arg in &files {
            let reader: Box<dyn BufRead> = if file_arg == "-" {
                Box::new(BufReader::new(io::stdin()))
            } else {
                match File::open(Path::new(file_arg)) {
                    Ok(f) => Box::new(BufReader::new(f)),
                    Err(e) => {
                        eprintln!("{}: can't open '{}': {}", applet_name, file_arg, e);
                        overall_success = false;
                        continue;
                    }
                }
            };

            let mut count_total = 0;
            let mut count_failed = 0;

            for line_res in reader.lines() {
                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => {
                        overall_success = false;
                        break;
                    }
                };

                let trimmed = line.trim_end_matches(&['\n', '\n'][..]);
                if trimmed.is_empty() {
                    continue;
                }

                let space_pos = match trimmed.find(' ') {
                    Some(pos) => pos,
                    None => {
                        if warn {
                            eprintln!("{}: invalid format", applet_name);
                        }
                        count_total += 1;
                        count_failed += 1;
                        overall_success = false;
                        continue;
                    }
                };

                let expected_hash = &trimmed[..space_pos];
                let mut filename_part = &trimmed[space_pos + 1..];
                if filename_part.starts_with(' ') || filename_part.starts_with('*') {
                    filename_part = &filename_part[1..];
                }

                count_total += 1;

                match hash_file_or_stdin(hash_type, applet_name, filename_part) {
                    Ok(actual_hash) => {
                        if actual_hash.eq_ignore_ascii_case(expected_hash) {
                            if !silent {
                                println!("{}: OK", filename_part);
                            }
                        } else {
                            if !silent {
                                println!("{}: FAILED", filename_part);
                            }
                            count_failed += 1;
                            overall_success = false;
                        }
                    }
                    Err(_) => {
                        if !silent {
                            println!("{}: FAILED", filename_part);
                        }
                        count_failed += 1;
                        overall_success = false;
                    }
                }
            }

            if count_failed > 0 && !silent {
                eprintln!(
                    "{}: WARNING: {} of {} computed checksums did NOT match",
                    applet_name, count_failed, count_total
                );
            }

            if count_total == 0 {
                eprintln!("{}: {}: no checksum lines found", applet_name, file_arg);
                overall_success = false;
            }
        }

        if overall_success {
            Ok(0)
        } else {
            Ok(1)
        }
    } else {
        let mut overall_success = true;
        let prefix = if binary_flag { "*" } else { " " };

        for file_arg in &files {
            match hash_file_or_stdin(hash_type, applet_name, file_arg) {
                Ok(hex) => {
                    println!("{} {}{}", hex, prefix, file_arg);
                }
                Err(_) => {
                    overall_success = false;
                }
            }
        }

        if overall_success {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}

pub fn parse_octal(b: &[u8]) -> Option<u32> {
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

pub fn split_user_group(spec: &[u8]) -> (&[u8], Option<&[u8]>) {
    match spec.iter().position(|&c| c == b':' || c == b'.') {
        Some(i) => (&spec[..i], Some(&spec[i + 1..])),
        None => (spec, None),
    }
}

pub fn parse_id_num(b: &[u8]) -> Option<u32> {
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

pub fn resolve_uid(name: &[u8]) -> Option<u32> {
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

pub fn resolve_gid(name: &[u8]) -> Option<u32> {
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

pub fn uid_name(uid: u32) -> Vec<u8> {
    let p = unsafe { libc::getpwuid(uid) };
    if p.is_null() {
        let mut b = Vec::new();
        put_num(&mut b, uid as u64);
        return b;
    }
    unsafe { CStr::from_ptr((*p).pw_name) }.to_bytes().to_vec()
}

pub fn gid_name(gid: u32) -> Vec<u8> {
    let p = unsafe { libc::getgrgid(gid) };
    if p.is_null() {
        let mut b = Vec::new();
        put_num(&mut b, gid as u64);
        return b;
    }
    unsafe { CStr::from_ptr((*p).gr_name) }.to_bytes().to_vec()
}

pub fn path_cstr(p: &Path) -> Option<CString> {
    CString::new(p.as_os_str().as_bytes()).ok()
}

pub fn lstat_of(p: &Path, follow: bool) -> Option<libc::stat> {
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

pub fn put_oct(buf: &mut Vec<u8>, mut n: u64) {
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
    put_num(buf, bytes / div);
    buf.push(b'.');
    buf.push(b'0' + ((bytes % div) * 10 / div) as u8);
    buf.push(U[ui]);
}

pub fn put_mode_str(buf: &mut Vec<u8>, mode: u32) {
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

pub fn file_type_char(mode: u32) -> u8 {
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

pub fn do_chmod(p: &Path, mode: u32) -> io::Result<()> {
    let c = path_cstr(p).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "bad path"))?;
    if unsafe { libc::chmod(c.as_ptr(), mode) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn chmod_tree(p: &Path, mode: u32) -> i32 {
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

pub fn do_chown(p: &Path, uid: u32, gid: u32, noderef: bool) -> io::Result<()> {
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

pub fn chown_tree(p: &Path, uid: u32, gid: u32, noderef: bool) -> i32 {
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

#[derive(Default)]
pub struct DuOpts {
    pub summarize: bool,
    pub all: bool,
    pub human: bool,
    pub apparent: bool,
    pub total: bool,
    pub one_fs: bool,
    pub follow: bool,
    pub div: u64,
    pub maxdepth: Option<u32>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OdType {
    Oct,
    Hex,
    Dec,
    Char,
    Named,
}

pub fn od_addr(buf: &mut Vec<u8>, off: u64, radix: u8) {
    let mut digs = [0u8; 24];
    let mut nd = 0;
    let mut v = off;
    if v == 0 {
        digs[0] = b'0';
        nd = 1;
    } else {
        let base = radix as u64;
        while v > 0 {
            let rem = (v % base) as u8;
            digs[nd] = if rem < 10 { b'0' + rem } else { b'a' + (rem - 10) };
            nd += 1;
            v /= base;
        }
    }
    let pad = match radix {
        8 => 7,
        16 => 6,
        10 => 7,
        _ => 7,
    };
    for _ in nd..pad {
        buf.push(b'0');
    }
    for i in (0..nd).rev() {
        buf.push(digs[i]);
    }
}

pub fn read_bytes_line(r: &mut impl BufRead, buf: &mut Vec<u8>) -> io::Result<bool> {
    buf.clear();
    let n = r.read_until(b'\n', buf)?;
    if n == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    Ok(true)
}

pub fn unicode_strwidth(s: &[u8]) -> usize {
    let mut width = 0;
    let mut i = 0;
    while i < s.len() {
        let b = s[i];
        if (b & 0xc0) != 0x80 {
            width += 1;
        }
        i += 1;
    }
    width
}


pub fn du_bytes(p: &Path, follow: bool, apparent: bool, one_fs: bool, top_dev: u64) -> u64 {
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

pub enum PasteInput {
    File(BufReader<std::fs::File>),
    Stdin,
}

pub fn next_line_paste(
    input: &mut PasteInput,
    stdin_lock: &mut io::StdinLock,
    buf: &mut Vec<u8>,
) -> Result<bool> {
    buf.clear();
    let n = match input {
        PasteInput::File(reader) => reader.read_until(b'\n', buf)?,
        PasteInput::Stdin => stdin_lock.read_until(b'\n', buf)?,
    };
    if n == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    Ok(true)
}

pub fn open_input(path: &Path) -> Result<Box<dyn BufRead>> {
    Ok(Box::new(BufReader::new(open_or_stdin(path)?)))
}

pub fn next_line(r: &mut Box<dyn BufRead>, buf: &mut Vec<u8>) -> Result<bool> {
    buf.clear();
    if r.read_until(b'\n', buf)? == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    Ok(true)
}

pub fn parse_delims(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() {
            i += 1;
            out.push(match raw[i] {
                b't' => b'\t',
                b'n' => b'\n',
                b'r' => b'\n',
                b'0' => b' ',
                c => c,
            });
        } else {
            out.push(raw[i]);
        }
        i += 1;
    }
    out
}

pub fn take_val(b: &[u8], j: usize, i: &mut usize, args: &[OsString]) -> Vec<u8> {
    if j + 1 < b.len() {
        b[j + 1..].to_vec()
    } else if *i + 1 < args.len() {
        *i += 1;
        args[*i].as_bytes().to_vec()
    } else {
        Vec::new()
    }
}

pub fn convert_stream<R: BufRead, W: Write>(mut r: R, mut w: W, to_unix: bool) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    let (mut pend, mut pcr) = (false, false);
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut out = Vec::with_capacity(n + 16);
        for &b in &buf[..n] {
            if to_unix {
                if b == b'\n' {
                    pcr = true;
                } else {
                    if pcr && b != b'\n' {
                        out.push(b'\n');
                    }
                    pcr = false;
                    out.push(b);
                }
            } else {
                if b == b'\n' {
                    if !pend {
                        out.push(b'\n');
                    }
                    out.push(b'\n');
                    pend = false;
                } else {
                    if b == b'\n' {
                        pend = true;
                    } else {
                        pend = false;
                    }
                    out.push(b);
                }
            }
        }
        w.write_all(&out)?;
    }
    if to_unix && pcr {
        w.write_all(b"
")?;
    }
    Ok(())
}

pub fn convert_file(path: &Path, to_unix: bool, tag: &str) -> Result<i32> {
    let tmp = path.with_extension("bb-tmp");
    let r: io::Result<()> = (|| {
        convert_stream(
            BufReader::new(std::fs::File::open(path)?),
            std::fs::File::create(&tmp)?,
            to_unix,
        )?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    })();
    match r {
        Ok(_) => Ok(0),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            eprintln!("{}: {}: {}", tag, path.display(), e);
            Ok(1)
        }
    }
}
