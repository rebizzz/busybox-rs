use crate::applets::archival::common::*;
use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct TarApplet;
impl Applet for TarApplet {
    fn name(&self) -> &'static str {
        "tar"
    }
    fn description(&self) -> &'static str {
        "Create, list or extract ustar archives (no compression)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = 0u8;
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
        Box::new(File::create(archive.unwrap())?)
    };

    let mut out = out;
    let mut rc = 0;
    for m in members {
        if let Err(e) = tar_append(&mut *out, m, m, verbose) {
            eprintln!("tar: {}: {}", m.display(), e);
            rc = 1;
        }
    }
    out.write_all(&[0u8; 1024])?;
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
            break;
        }

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
                continue;
            }
        }

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
