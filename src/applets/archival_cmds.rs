//! Minimal archival/compression batch: tar, cpio, gzip/gunzip/uncompress, cksum.
//! std+libc only. No deflate/zlib: gzip paths are stored/copy pass-through
//! stubs that say so. Upstream behavior notes: tar (ustar, -c/-x/-t/-f/-v/-C),
//! cpio (newc only, no CRC check), gzip (stdin/stdout + file operands).

use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// ---- shared tiny helpers (size discipline: callers use 512/8K stack bufs) ---

/// Write `val` as octal ASCII into `buf[off..off+width]`, NUL-terminated.
fn write_octal(buf: &mut [u8], off: usize, width: usize, val: u64) {
    let s = format!("{:0>1$o}", val, width - 1);
    let b = s.as_bytes();
    let n = b.len().min(width - 1);
    buf[off..off + n].copy_from_slice(&b[b.len() - n..]);
    buf[off + width - 1] = 0;
}

fn parse_octal(b: &[u8]) -> u64 {
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

fn parse_hex8(b: &[u8]) -> u64 {
    let s = std::str::from_utf8(b).unwrap_or("");
    u64::from_str_radix(s.trim_matches('\0').trim(), 16).unwrap_or(0)
}

fn copy_stream<R: Read + ?Sized, W: Write + ?Sized>(r: &mut R, w: &mut W) -> std::io::Result<u64> {
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

/// Single 256-entry CRC-32 table (POSIX cksum, poly 0x04C11DB7), built on stack.
fn build_crc_table() -> [u32; 256] {
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

fn cksum_update(tab: &[u32; 256], mut crc: u32, data: &[u8]) -> u32 {
    for &b in data {
        crc = tab[((crc >> 24) as u8 ^ b) as usize] ^ (crc << 8);
    }
    crc
}

// ---- tar (ustar, no compression) ----

pub struct TarApplet;
impl Applet for TarApplet {
    fn name(&self) -> &'static str {
        "tar"
    }
    fn description(&self) -> &'static str {
        "Create, list or extract ustar archives (no compression)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = 0u8; // 1=c 2=x 3=t
        let mut archive: Option<PathBuf> = None;
        let mut chdir: Option<PathBuf> = None;
        let mut verbose = false;
        let mut members: Vec<PathBuf> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-f" || b == b"--file" {
                i += 1;
                if i >= args.len() {
                    eprintln!("tar: -f requires an argument");
                    return Ok(1);
                }
                archive = Some(PathBuf::from(&args[i]));
            } else if b.starts_with(b"-") && b.len() > 1 && !b.starts_with(b"--") {
                // expand bundled shorts: tar -cvf a.tar dir
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b'c' => mode = 1,
                        b'x' => mode = 2,
                        b't' => mode = 3,
                        b'v' => verbose = true,
                        b'f' => {
                            let rest = &b[j + 1..];
                            if !rest.is_empty() {
                                archive = Some(PathBuf::from(std::ffi::OsStr::from_bytes(rest)));
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("tar: -f requires an argument");
                                    return Ok(1);
                                }
                                archive = Some(PathBuf::from(&args[i]));
                            }
                            break;
                        }
                        b'C' => {
                            i += 1;
                            if i >= args.len() {
                                eprintln!("tar: -C requires an argument");
                                return Ok(1);
                            }
                            chdir = Some(PathBuf::from(&args[i]));
                            break;
                        }
                        b'z' | b'j' | b'J' => {
                            eprintln!(
                                "tar: compression not supported (no zlib); use -c/-x on plain tar"
                            );
                            return Ok(1);
                        }
                        _ => {}
                    }
                    j += 1;
                }
            } else if b == b"-C" {
                i += 1;
                if i >= args.len() {
                    eprintln!("tar: -C requires an argument");
                    return Ok(1);
                }
                chdir = Some(PathBuf::from(&args[i]));
            } else {
                members.push(PathBuf::from(&args[i]));
            }
            i += 1;
        }
        if mode == 0 {
            eprintln!("tar: need one of -c/-x/-t");
            return Ok(1);
        }
        if let Some(d) = chdir {
            if let Err(e) = std::env::set_current_dir(&d) {
                eprintln!("tar: cannot chdir: {}", e);
                return Ok(1);
            }
        }
        match mode {
            1 => tar_create(archive.as_deref(), &members, verbose),
            2 => tar_list_or_extract(archive.as_deref(), &members, verbose, true),
            _ => tar_list_or_extract(archive.as_deref(), &members, verbose, false),
        }
    }
}

fn tar_header(path: &str, mode: u32, size: u64, mtime: u64, typeflag: u8) -> [u8; 512] {
    let mut h = [0u8; 512];
    let name = path.as_bytes();
    let n = name.len().min(100);
    h[..n].copy_from_slice(&name[..n]);
    write_octal(&mut h, 100, 8, mode as u64);
    write_octal(&mut h, 108, 8, 0);
    write_octal(&mut h, 116, 8, 0);
    write_octal(&mut h, 124, 12, size);
    write_octal(&mut h, 136, 12, mtime);
    h[156] = typeflag;
    h[257..262].copy_from_slice(b"ustar");
    h[263..265].copy_from_slice(b"00");
    // checksum with spaces
    let mut sum = 0u64;
    for &c in &h {
        sum += c as u64;
    }
    sum += 8 * (b' ' as u64);
    let s = format!("{:06o}\0 ", sum);
    h[148..156].copy_from_slice(&s.as_bytes()[..8]);
    h
}

fn tar_create(archive: Option<&Path>, members: &[PathBuf], verbose: bool) -> Result<i32> {
    let out: Box<dyn Write> = if archive.map(|p| p.as_os_str() == "-").unwrap_or(true) {
        Box::new(std::io::stdout())
    } else {
        // `map` above returned false, so `archive` is Some(non-dash path)
        Box::new(File::create(archive.unwrap())?)
    };
    // need Write without Box overhead issues; use concrete via helper closure
    let mut out = out;
    let mut rc = 0;
    for m in members {
        if let Err(e) = tar_append(&mut *out, m, m, verbose) {
            eprintln!("tar: {}: {}", m.display(), e);
            rc = 1;
        }
    }
    out.write_all(&[0u8; 1024])?; // two zero blocks
    out.flush()?;
    Ok(rc)
}

fn tar_append(
    out: &mut dyn Write,
    fs_path: &Path,
    arc_path: &Path,
    verbose: bool,
) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(fs_path)?;
    let arc = arc_path.to_string_lossy().replace('\\', "/");
    if meta.is_dir() {
        let name = if arc.ends_with('/') { arc } else { arc + "/" };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.write_all(&tar_header(&name, 0o755, 0, mtime, b'5'))?;
        if verbose {
            println!("{}", name);
        }
        let mut entries: Vec<_> = std::fs::read_dir(fs_path)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            tar_append(out, &e.path(), &arc_path.join(e.file_name()), verbose)?;
        }
    } else if meta.is_file() {
        let mode = meta.permissions().mode() & 0o777;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.write_all(&tar_header(&arc, mode, meta.len(), mtime, b'0'))?;
        if verbose {
            println!("{}", arc);
        }
        let mut f = File::open(fs_path)?;
        let mut buf = [0u8; 8192];
        let mut left = meta.len();
        while left > 0 {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
            left -= n as u64;
        }
        let pad = (512 - (meta.len() % 512)) % 512;
        if pad > 0 {
            out.write_all(&vec![0u8; pad as usize])?;
        }
    } else {
        eprintln!("tar: skipping non-regular file {}", fs_path.display());
    }
    Ok(())
}

fn tar_full_name(h: &[u8]) -> String {
    let prefix = std::str::from_utf8(&h[345..500])
        .unwrap_or("")
        .trim_matches('\0');
    let name = std::str::from_utf8(&h[..100])
        .unwrap_or("")
        .trim_matches('\0');
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", prefix, name)
    }
}

fn tar_list_or_extract(
    archive: Option<&Path>,
    filter: &[PathBuf],
    verbose: bool,
    extract: bool,
) -> Result<i32> {
    let path = archive.unwrap_or(Path::new("-"));
    let mut inp = open_or_stdin(path)?;
    let wanted = |n: &str| filter.is_empty() || filter.iter().any(|f| f.to_string_lossy() == n);
    let mut hdr = [0u8; 512];
    let mut rc = 0;
    loop {
        // read full 512-byte block
        let mut got = 0;
        while got < 512 {
            match inp.read(&mut hdr[got..])? {
                0 => break,
                n => got += n,
            }
        }
        if got == 0 {
            break;
        }
        if got < 512 {
            eprintln!("tar: truncated header");
            return Ok(1);
        }
        if hdr.iter().all(|&b| b == 0) {
            break; // end (require one zero block; tolerate single)
        }
        // verify checksum best-effort
        let stored = parse_octal(&hdr[148..156]);
        let mut sum = 8 * (b' ' as u64);
        for (k, &c) in hdr.iter().enumerate() {
            if !(148..156).contains(&k) {
                sum += c as u64;
            }
        }
        let name = tar_full_name(&hdr);
        if sum != stored {
            eprintln!("tar: bad checksum for {}", name);
            rc = 1;
            // still try to skip
        }
        let size = parse_octal(&hdr[124..136]);
        let typeflag = hdr[156];
        let is_dir = typeflag == b'5' || name.ends_with('/');
        if !extract {
            if wanted(&name) {
                if verbose {
                    println!("{:>8} {}", size, name);
                } else {
                    println!("{}", name);
                }
            }
        } else if wanted(&name) {
            if verbose {
                println!("{}", name);
            }
            if is_dir {
                if let Err(e) = std::fs::create_dir_all(&name) {
                    eprintln!("tar: {}: {}", name, e);
                    rc = 1;
                }
            } else if typeflag == b'0' || typeflag == 0 {
                if let Some(p) = Path::new(&name).parent() {
                    if !p.as_os_str().is_empty() {
                        let _ = std::fs::create_dir_all(p);
                    }
                }
                match File::create(&name) {
                    Ok(mut f) => {
                        let mut left = size;
                        let mut buf = [0u8; 8192];
                        while left > 0 {
                            let want = (left as usize).min(buf.len());
                            let mut got2 = 0;
                            while got2 < want {
                                match inp.read(&mut buf[got2..want])? {
                                    0 => break,
                                    n => got2 += n,
                                }
                            }
                            if got2 == 0 {
                                eprintln!("tar: truncated file {}", name);
                                rc = 1;
                                break;
                            }
                            if f.write_all(&buf[..got2]).is_err() {
                                rc = 1;
                                break;
                            }
                            left -= got2 as u64;
                        }
                        let mode = parse_octal(&hdr[100..108]) as u32 & 0o777;
                        let _ =
                            std::fs::set_permissions(&name, std::fs::Permissions::from_mode(mode));
                        // skip padding without reading into output
                        let pad = (512 - (size % 512)) % 512;
                        let mut skip = pad;
                        let mut tmp = [0u8; 512];
                        while skip > 0 {
                            match inp.read(&mut tmp[..(skip as usize).min(512)])? {
                                0 => break,
                                n => skip -= n as u64,
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("tar: {}: {}", name, e);
                        rc = 1;
                    }
                }
                continue; // data+pad already consumed
            }
        }
        // skip data blocks for list / non-wanted / dirs
        let skip_blocks = size.div_ceil(512);
        let mut to_skip = skip_blocks * 512;
        let mut tmp = [0u8; 8192];
        while to_skip > 0 {
            let want = (to_skip as usize).min(tmp.len());
            match inp.read(&mut tmp[..want])? {
                0 => break,
                n => to_skip -= n as u64,
            }
        }
    }
    Ok(rc)
}

// ---- cpio (newc read/write) ----

pub struct CpioApplet;
impl Applet for CpioApplet {
    fn name(&self) -> &'static str {
        "cpio"
    }
    fn description(&self) -> &'static str {
        "Create/extract cpio newc archives"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut create = false;
        let mut extract = false;
        let mut list = false;
        let mut verbose = false;
        let mut arch: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-F" {
                i += 1;
                if i >= args.len() {
                    eprintln!("cpio: -F requires an argument");
                    return Ok(1);
                }
                arch = Some(PathBuf::from(&args[i]));
            } else if b.starts_with(b"-") && b.len() > 1 {
                for &c in &b[1..] {
                    match c {
                        b'o' => create = true,
                        b'i' => extract = true,
                        b't' => list = true,
                        b'v' => verbose = true,
                        _ => {}
                    }
                }
            }
            i += 1;
        }
        if create {
            cpio_create(arch.as_deref(), verbose)
        } else if extract || list {
            cpio_extract(arch.as_deref(), verbose, extract)
        } else {
            eprintln!("cpio: need -o (create) or -i/-t (extract/list)");
            Ok(1)
        }
    }
}

fn cpio_write_header(
    out: &mut dyn Write,
    name: &str,
    mode: u32,
    size: u64,
    mtime: u64,
) -> std::io::Result<()> {
    // magic + 13 x 8-hex fields; namesize includes NUL
    let hdr = format!(
        "070701{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}",
        0u32,
        mode,
        0u32,
        0u32,
        1u32,
        mtime as u32,
        size as u32,
        0u32,
        0u32,
        0u32,
        0u32,
        name.len() as u32 + 1,
        0u32
    );
    out.write_all(hdr.as_bytes())?;
    out.write_all(name.as_bytes())?;
    out.write_all(b"\0")?;
    let pad = (4 - (110 + name.len() + 1) % 4) % 4;
    out.write_all(&vec![0u8; pad])?;
    Ok(())
}

fn cpio_create(arch: Option<&Path>, verbose: bool) -> Result<i32> {
    // file list: names on stdin (one per line), like `find . | cpio -o`
    let list_bytes = {
        let mut inp = open_or_stdin(Path::new("-"))?;
        let mut v = Vec::new();
        inp.read_to_end(&mut v)?;
        v
    };
    let out: Box<dyn Write> = match arch {
        None => Box::new(std::io::stdout()),
        Some(p) => Box::new(File::create(p)?),
    };
    let mut out = out;
    let mut rc = 0;
    for line in list_bytes.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let os = std::ffi::OsStr::from_bytes(line);
        let p = Path::new(os);
        let meta = match std::fs::symlink_metadata(p) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("cpio: {}: {}", p.display(), e);
                rc = 1;
                continue;
            }
        };
        let name = p.to_string_lossy().into_owned();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if meta.is_dir() {
            cpio_write_header(&mut *out, &name, 0o040755, 0, mtime)?;
        } else if meta.is_file() {
            let mode = 0o100000 | (meta.permissions().mode() & 0o777);
            cpio_write_header(&mut *out, &name, mode, meta.len(), mtime)?;
            let mut f = File::open(p)?;
            copy_stream(&mut f, &mut *out)?;
            let pad = (4 - (meta.len() % 4)) % 4;
            if pad > 0 {
                out.write_all(&vec![0u8; pad as usize])?;
            }
        } else {
            eprintln!("cpio: skipping non-regular file {}", p.display());
            continue;
        }
        if verbose {
            println!("{}", name);
        }
    }
    cpio_write_header(&mut *out, "TRAILER!!!", 0, 0, 0)?;
    out.flush()?;
    Ok(rc)
}

fn cpio_read_exact<R: Read + ?Sized>(
    r: &mut R,
    n: usize,
    buf: &mut Vec<u8>,
) -> std::io::Result<bool> {
    buf.resize(n, 0);
    let mut got = 0;
    while got < n {
        match r.read(&mut buf[got..n])? {
            0 => {
                return Ok(false);
            }
            k => got += k,
        }
    }
    Ok(true)
}

fn cpio_extract(arch: Option<&Path>, verbose: bool, do_extract: bool) -> Result<i32> {
    let path = arch.unwrap_or(Path::new("-"));
    let mut inp = open_or_stdin(path)?;
    let mut rc = 0;
    let mut hdr = Vec::new();
    let mut skip = vec![0u8; 8192];
    loop {
        if !cpio_read_exact(&mut *inp, 110, &mut hdr)? {
            break;
        }
        if &hdr[..6] != b"070701" && &hdr[..6] != b"070702" {
            eprintln!("cpio: bad magic (only newc supported)");
            return Ok(1);
        }
        let filesize = parse_hex8(&hdr[54..62]);
        let namesize = parse_hex8(&hdr[94..102]) as usize;
        if namesize == 0 || namesize > 4096 {
            eprintln!("cpio: bad namesize");
            return Ok(1);
        }
        let mut namebuf = Vec::new();
        if !cpio_read_exact(&mut *inp, namesize, &mut namebuf)? {
            eprintln!("cpio: truncated name");
            return Ok(1);
        }
        let pad1 = (4 - (110 + namesize) % 4) % 4;
        if pad1 > 0 {
            let mut p = vec![0u8; pad1];
            let _ = cpio_read_exact(&mut *inp, pad1, &mut p);
        }
        let raw = &namebuf[..namesize.saturating_sub(1)];
        let name = String::from_utf8_lossy(raw).into_owned();
        if name == "TRAILER!!!" {
            break;
        }
        let mode = parse_hex8(&hdr[14..22]) as u32;
        if verbose || !do_extract {
            println!("{}", name);
        }
        if do_extract {
            if mode & 0o040000 == 0o040000 {
                if let Err(e) = std::fs::create_dir_all(&name) {
                    eprintln!("cpio: {}: {}", name, e);
                    rc = 1;
                }
            } else {
                if let Some(p) = Path::new(&name).parent() {
                    if !p.as_os_str().is_empty() {
                        let _ = std::fs::create_dir_all(p);
                    }
                }
                match File::create(&name) {
                    Ok(mut f) => {
                        let mut left = filesize;
                        while left > 0 {
                            let want = (left as usize).min(skip.len());
                            let mut got = 0;
                            while got < want {
                                match inp.read(&mut skip[got..want])? {
                                    0 => break,
                                    n => got += n,
                                }
                            }
                            if got == 0 {
                                eprintln!("cpio: truncated file {}", name);
                                rc = 1;
                                break;
                            }
                            if f.write_all(&skip[..got]).is_err() {
                                rc = 1;
                                break;
                            }
                            left -= got as u64;
                        }
                        let _ = std::fs::set_permissions(
                            &name,
                            std::fs::Permissions::from_mode(mode & 0o777),
                        );
                    }
                    Err(e) => {
                        eprintln!("cpio: {}: {}", name, e);
                        rc = 1;
                    }
                }
            }
        } else {
            // list mode: skip data
            let mut left = filesize;
            while left > 0 {
                let want = (left as usize).min(skip.len());
                match inp.read(&mut skip[..want])? {
                    0 => break,
                    n => left -= n as u64,
                }
            }
        }
        let pad2 = (4 - (filesize % 4)) % 4;
        if pad2 > 0 {
            let mut p = vec![0u8; pad2 as usize];
            let _ = cpio_read_exact(&mut *inp, pad2 as usize, &mut p);
        }
    }
    Ok(rc)
}

// ---- gzip / gunzip / uncompress (stored pass-through stubs) ----

fn gzip_copy(in_path: &Path, out: &mut dyn Write) -> Result<(u64, bool)> {
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

pub struct GzipApplet;
impl Applet for GzipApplet {
    fn name(&self) -> &'static str {
        "gzip"
    }
    fn description(&self) -> &'static str {
        "Copy input to output (stored only; no zlib)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // NOTE: no deflate available (std+libc only); stored pass-through.
        let mut to_stdout = false;
        let mut files: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-c" || b == b"--stdout" || b == b"--to-stdout" {
                to_stdout = true;
            } else if b.starts_with(b"-") && b.len() > 1 && !b.starts_with(b"--") {
                if b.iter()
                    .all(|&c| matches!(c, b'c' | b'f' | b'k' | b'v' | b'1'..=b'9' | b'd'))
                {
                    if b.contains(&b'c') {
                        to_stdout = true;
                    }
                } else {
                    files.push(Path::new(a));
                }
            } else {
                files.push(Path::new(a));
            }
        }
        eprintln!("gzip: no zlib in this build; copying without compression");
        if files.is_empty() {
            let mut so = std::io::stdout().lock();
            gzip_copy(Path::new("-"), &mut so)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            let dest: PathBuf = if to_stdout {
                PathBuf::from("-")
            } else {
                let mut s = f.as_os_str().as_bytes().to_vec();
                s.extend_from_slice(b".gz");
                PathBuf::from(std::ffi::OsStr::from_bytes(&s))
            };
            let res: Result<()> = (|| {
                if dest.as_os_str() == "-" {
                    let mut so = std::io::stdout().lock();
                    gzip_copy(f, &mut so)?;
                } else if let Err(e) = (|| -> Result<()> {
                    let mut o = File::create(&dest)?;
                    let _ = gzip_copy(f, &mut o)?;
                    Ok(())
                })() {
                    eprintln!("gzip: {}: {}", f.display(), e);
                    rc = 1;
                }
                Ok(())
            })();
            let _ = res;
        }
        Ok(rc)
    }
}

pub struct GunzipApplet;
impl Applet for GunzipApplet {
    fn name(&self) -> &'static str {
        "gunzip"
    }
    fn description(&self) -> &'static str {
        "Copy input to output (no zlib inflate)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut to_stdout = false;
        let mut files: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-c" || b == b"--stdout" || b == b"--to-stdout" {
                to_stdout = true;
            } else if b.starts_with(b"-") && !b.starts_with(b"--") && b.len() > 1 {
                if b.contains(&b'c') {
                    to_stdout = true;
                }
            } else {
                files.push(Path::new(a));
            }
        }
        if files.is_empty() {
            let mut so = std::io::stdout().lock();
            let (_, is_gz) = gzip_copy(Path::new("-"), &mut so)?;
            if is_gz {
                eprintln!("gunzip: compressed data requires zlib (not compiled in)");
                return Ok(1);
            }
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            let dest: PathBuf = if to_stdout {
                PathBuf::from("-")
            } else {
                let raw = f.as_os_str().as_bytes();
                let s: &[u8] = raw
                    .strip_suffix(b".gz")
                    .or_else(|| raw.strip_suffix(b".tgz"))
                    .or_else(|| raw.strip_suffix(b".Z"))
                    .unwrap_or(raw);
                PathBuf::from(std::ffi::OsStr::from_bytes(s))
            };
            let r: Result<(u64, bool)> = (|| {
                if dest.as_os_str() == "-" {
                    let mut so = std::io::stdout().lock();
                    Ok(gzip_copy(f, &mut so)?)
                } else {
                    let mut o = File::create(&dest)?;
                    Ok(gzip_copy(f, &mut o)?)
                }
            })();
            match r {
                Ok((_, true)) => {
                    eprintln!(
                        "gunzip: {}: compressed data requires zlib (not compiled in)",
                        f.display()
                    );
                    rc = 1;
                }
                Err(e) => {
                    eprintln!("gunzip: {}: {}", f.display(), e);
                    rc = 1;
                }
                Ok(_) => {}
            }
        }
        Ok(rc)
    }
}

pub struct UncompressAlias;
impl Applet for UncompressAlias {
    fn name(&self) -> &'static str {
        "uncompress"
    }
    fn description(&self) -> &'static str {
        "Alias for gunzip (no zlib inflate)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        GunzipApplet.run(args)
    }
}

// ---- cksum (POSIX CRC32) ----

pub struct CksumApplet;
impl Applet for CksumApplet {
    fn name(&self) -> &'static str {
        "cksum"
    }
    fn description(&self) -> &'static str {
        "Print POSIX CRC32 checksum and size"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let tab = build_crc_table();
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            let (crc, len) = cksum_stream(&tab, Path::new("-"))?;
            println!("{} {}", crc, len);
            return Ok(0);
        }
        let mut rc = 0;
        for f in files {
            match cksum_stream(&tab, f) {
                Ok((crc, len)) => {
                    if f.as_os_str() == "-" {
                        println!("{} {}", crc, len);
                    } else {
                        println!("{} {} {}", crc, len, f.display());
                    }
                }
                Err(e) => {
                    eprintln!("cksum: {}: {}", f.display(), e);
                    rc = 1;
                }
            }
        }
        Ok(rc)
    }
}

fn cksum_stream(tab: &[u32; 256], path: &Path) -> Result<(u32, u64)> {
    let mut inp = open_or_stdin(path)?;
    let mut buf = [0u8; 8192];
    let mut crc = 0u32;
    let mut len = 0u64;
    loop {
        let n = inp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        crc = cksum_update(tab, crc, &buf[..n]);
        len += n as u64;
    }
    // fold length (POSIX)
    let mut l = len;
    while l > 0 {
        crc = cksum_update(tab, crc, &[((l & 0xff) as u8)]);
        l >>= 8;
    }
    Ok((!crc, len))
}
