use crate::applets::archival::common::*;
use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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
        if !cpio_read_exact(&mut *inp, 2, &mut hdr)? {
            break;
        }
        let is_bin_le = hdr[0] == 0xc7 && hdr[1] == 0x71;
        let is_bin_be = hdr[0] == 0x71 && hdr[1] == 0xc7;
        let (filesize, namesize, hdr_len, pad_align, mode) = if is_bin_le || is_bin_be {
            let mut rest = Vec::new();
            if !cpio_read_exact(&mut *inp, 24, &mut rest)? {
                break;
            }
            hdr.extend_from_slice(&rest);
            let m = if is_bin_le {
                u16::from_le_bytes([hdr[6], hdr[7]]) as u32
            } else {
                u16::from_be_bytes([hdr[6], hdr[7]]) as u32
            };
            let ns = if is_bin_le {
                u16::from_le_bytes([hdr[18], hdr[19]]) as usize
            } else {
                u16::from_be_bytes([hdr[18], hdr[19]]) as usize
            };
            let fs = if is_bin_le {
                ((u16::from_le_bytes([hdr[20], hdr[21]]) as u64) << 16)
                    | (u16::from_le_bytes([hdr[22], hdr[23]]) as u64)
            } else {
                ((u16::from_be_bytes([hdr[20], hdr[21]]) as u64) << 16)
                    | (u16::from_be_bytes([hdr[22], hdr[23]]) as u64)
            };
            (fs, ns, 26, 2, m)
        } else {
            let mut rest = Vec::new();
            if !cpio_read_exact(&mut *inp, 4, &mut rest)? {
                break;
            }
            hdr.extend_from_slice(&rest);
            if &hdr[..6] == b"070701" || &hdr[..6] == b"070702" {
                let mut rest_hdr = Vec::new();
                if !cpio_read_exact(&mut *inp, 104, &mut rest_hdr)? {
                    break;
                }
                hdr.extend_from_slice(&rest_hdr);
                let m = parse_hex8(&hdr[14..22]) as u32;
                let fs = parse_hex8(&hdr[54..62]);
                let ns = parse_hex8(&hdr[94..102]) as usize;
                (fs, ns, 110, 4, m)
            } else if &hdr[..6] == b"070707" {
                let mut rest_hdr = Vec::new();
                if !cpio_read_exact(&mut *inp, 70, &mut rest_hdr)? {
                    break;
                }
                hdr.extend_from_slice(&rest_hdr);
                let m = parse_octal(&hdr[18..24]) as u32;
                let fs = parse_octal(&hdr[44..55]);
                let ns = parse_octal(&hdr[59..65]) as usize;
                (fs, ns, 76, 1, m)
            } else {
                eprintln!("cpio: bad magic");
                return Ok(1);
            }
        };
        if namesize == 0 || namesize > 4096 {
            eprintln!("cpio: bad namesize");
            return Ok(1);
        }
        let mut namebuf = Vec::new();
        if !cpio_read_exact(&mut *inp, namesize, &mut namebuf)? {
            eprintln!("cpio: truncated name");
            return Ok(1);
        }
        let pad1 = (pad_align - (hdr_len + namesize) % pad_align) % pad_align;
        if pad1 > 0 {
            let mut p = vec![0u8; pad1];
            let _ = cpio_read_exact(&mut *inp, pad1, &mut p);
        }
        let raw = &namebuf[..namesize.saturating_sub(1)];
        let name = String::from_utf8_lossy(raw).into_owned();
        if name == "TRAILER!!!" {
            break;
        }
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
            let mut left = filesize;
            while left > 0 {
                let want = (left as usize).min(skip.len());
                match inp.read(&mut skip[..want])? {
                    0 => break,
                    n => left -= n as u64,
                }
            }
        }
        let pad2 = (pad_align - (filesize as usize % pad_align)) % pad_align;
        if pad2 > 0 {
            let mut p = vec![0u8; pad2];
            let _ = cpio_read_exact(&mut *inp, pad2, &mut p);
        }
    }
    Ok(rc)
}

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
