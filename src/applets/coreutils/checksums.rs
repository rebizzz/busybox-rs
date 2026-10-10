use crate::core::digest::{BsdSum, Digest, Md5, Sha1, Sha256, Sha512, SysVSum};
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
enum HashType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl HashType {
    fn create_digest(&self) -> Box<dyn Digest> {
        match self {
            HashType::Md5 => Box::new(Md5::new()),
            HashType::Sha1 => Box::new(Sha1::new()),
            HashType::Sha256 => Box::new(Sha256::new()),
            HashType::Sha512 => Box::new(Sha512::new()),
        }
    }
}

fn hash_stream<R: Read>(digest: &mut dyn Digest, mut reader: R) -> io::Result<()> {
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

fn hash_file_or_stdin(
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

fn run_hash_cmd(applet_name: &'static str, hash_type: HashType, args: &[OsString]) -> Result<i32> {
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

                let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
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

pub struct Md5SumApplet;
impl Applet for Md5SumApplet {
    fn name(&self) -> &'static str {
        "md5sum"
    }
    fn description(&self) -> &'static str {
        "Print or check MD5 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("md5sum", HashType::Md5, args)
    }
}

pub struct Sha1SumApplet;
impl Applet for Sha1SumApplet {
    fn name(&self) -> &'static str {
        "sha1sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA1 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha1sum", HashType::Sha1, args)
    }
}

pub struct Sha256SumApplet;
impl Applet for Sha256SumApplet {
    fn name(&self) -> &'static str {
        "sha256sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA256 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha256sum", HashType::Sha256, args)
    }
}

pub struct Sha512SumApplet;
impl Applet for Sha512SumApplet {
    fn name(&self) -> &'static str {
        "sha512sum"
    }
    fn description(&self) -> &'static str {
        "Print or check SHA512 checksums"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_hash_cmd("sha512sum", HashType::Sha512, args)
    }
}

pub struct SumApplet;
impl Applet for SumApplet {
    fn name(&self) -> &'static str {
        "sum"
    }
    fn description(&self) -> &'static str {
        "Checksum and count the blocks in a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sysv = false;
        let mut bsd = false;
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
                        's' => {
                            sysv = true;
                        }
                        'r' => {
                            bsd = true;
                        }
                        _ => {
                            eprintln!("sum: unrecognized option '{}'", s);
                            return Ok(1);
                        }
                    }
                }
                continue;
            }
            files.push(s.into_owned());
        }

        let is_sysv = sysv && !bsd;

        let num_files = files.len();
        let (actual_files, print_name) = if files.is_empty() {
            (vec!["-".to_string()], is_sysv)
        } else {
            let p_name = num_files > 1 || is_sysv;
            (files, p_name)
        };

        let mut overall_success = true;

        for file in &actual_files {
            let name_to_print = if print_name { file.as_str() } else { "" };

            if is_sysv {
                let mut hasher = SysVSum::new();
                let res = if file == "-" {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buf = [0u8; 8192];
                    loop {
                        match handle.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => hasher.update(&buf[..n]),
                            Err(e) => break Err(e),
                        }
                    }
                } else {
                    match File::open(Path::new(file)) {
                        Ok(mut f) => {
                            let mut buf = [0u8; 8192];
                            loop {
                                match f.read(&mut buf) {
                                    Ok(0) => break Ok(()),
                                    Ok(n) => hasher.update(&buf[..n]),
                                    Err(e) => break Err(e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("sum: can't open '{}': {}", file, e);
                            overall_success = false;
                            continue;
                        }
                    }
                };

                if let Err(e) = res {
                    eprintln!("sum: {}: {}", file, e);
                    overall_success = false;
                    continue;
                }

                let (sum, blocks) = hasher.finalize();
                println!("{} {} {}", sum, blocks, name_to_print);
            } else {
                let mut hasher = BsdSum::new();
                let res = if file == "-" {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buf = [0u8; 8192];
                    loop {
                        match handle.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => hasher.update(&buf[..n]),
                            Err(e) => break Err(e),
                        }
                    }
                } else {
                    match File::open(Path::new(file)) {
                        Ok(mut f) => {
                            let mut buf = [0u8; 8192];
                            loop {
                                match f.read(&mut buf) {
                                    Ok(0) => break Ok(()),
                                    Ok(n) => hasher.update(&buf[..n]),
                                    Err(e) => break Err(e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("sum: can't open '{}': {}", file, e);
                            overall_success = false;
                            continue;
                        }
                    }
                };

                if let Err(e) = res {
                    eprintln!("sum: {}: {}", file, e);
                    overall_success = false;
                    continue;
                }

                let (sum, blocks) = hasher.finalize();
                println!("{:05} {:5} {}", sum, blocks, name_to_print);
            }
        }

        if overall_success {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}
