use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
