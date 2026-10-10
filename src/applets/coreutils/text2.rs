use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
fn open_input(path: &Path) -> Result<Box<dyn BufRead>> {
    Ok(Box::new(BufReader::new(open_or_stdin(path)?)))
}
fn next_line(r: &mut Box<dyn BufRead>, buf: &mut Vec<u8>) -> Result<bool> {
    buf.clear();
    if r.read_until(b'\n', buf)? == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    Ok(true)
}
fn parse_delims(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() {
            i += 1;
            out.push(match raw[i] {
                b't' => b'\t',
                b'n' => b'\n',
                b'r' => b'\r',
                b'0' => b'\0',
                c => c,
            });
        } else {
            out.push(raw[i]);
        }
        i += 1;
    }
    out
}
fn take_val(b: &[u8], j: usize, i: &mut usize, args: &[OsString]) -> Vec<u8> {
    if j + 1 < b.len() {
        b[j + 1..].to_vec()
    } else if *i + 1 < args.len() {
        *i += 1;
        args[*i].as_bytes().to_vec()
    } else {
        Vec::new()
    }
}

enum PasteInput {
    File(BufReader<std::fs::File>),
    Stdin,
}

fn next_line_paste(
    input: &mut PasteInput,
    stdin_lock: &mut io::StdinLock,
    buf: &mut Vec<u8>,
) -> Result<bool> {
    buf.clear();
    let n = match input {
        PasteInput::File(r) => r.read_until(b'\n', buf)?,
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

pub struct PasteApplet;
impl Applet for PasteApplet {
    fn name(&self) -> &'static str {
        "paste"
    }
    fn description(&self) -> &'static str {
        "Merge lines of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delims = vec![b'\t'];
        let mut serial = false;
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1 && b[0] == b'-' && b != b"-" {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b's' => {
                            serial = true;
                            j += 1;
                        }
                        b'd' => {
                            let v = take_val(b, j, &mut i, args);
                            if v.is_empty() {
                                eprintln!("paste: no delimiters");
                                return Ok(1);
                            }
                            delims = parse_delims(&v);
                            break;
                        }
                        _ => j += 1,
                    }
                }
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut buf = Vec::with_capacity(4096);

        if serial {
            for f in &files {
                let mut input = if f.as_os_str() == "-" {
                    PasteInput::Stdin
                } else {
                    PasteInput::File(BufReader::new(std::fs::File::open(f)?))
                };
                let mut first = true;
                let mut di = 0;
                while next_line_paste(&mut input, &mut stdin_lock, &mut buf)? {
                    if !first {
                        let d = delims[di % delims.len()];
                        if d != 0 {
                            w.write_all(&[d])?;
                        }
                        di += 1;
                    }
                    first = false;
                    w.write_all(&buf)?;
                }
                if !first {
                    w.write_all(b"\n")?;
                }
            }
        } else {
            let mut inputs: Vec<Option<PasteInput>> = Vec::with_capacity(files.len());
            for f in &files {
                let input = if f.as_os_str() == "-" {
                    PasteInput::Stdin
                } else {
                    PasteInput::File(BufReader::new(std::fs::File::open(f)?))
                };
                inputs.push(Some(input));
            }

            let mut lines: Vec<Option<Vec<u8>>> = vec![None; inputs.len()];
            loop {
                let mut any = false;
                for k in 0..inputs.len() {
                    if let Some(ref mut inp) = inputs[k] {
                        if next_line_paste(inp, &mut stdin_lock, &mut buf)? {
                            any = true;
                            lines[k] = Some(buf.clone());
                        } else {
                            inputs[k] = None;
                            lines[k] = None;
                        }
                    } else {
                        lines[k] = None;
                    }
                }
                if !any {
                    break;
                }
                let mut del_idx = 0;
                for k in 0..inputs.len() {
                    if let Some(ref line_buf) = lines[k] {
                        w.write_all(line_buf)?;
                    }
                    let delim = if k == inputs.len() - 1 {
                        b'\n'
                    } else {
                        let d = delims[del_idx];
                        del_idx = (del_idx + 1) % delims.len();
                        d
                    };
                    if delim != 0 {
                        w.write_all(&[delim])?;
                    }
                }
            }
        }
        Ok(0)
    }
}

pub struct NlApplet;
impl Applet for NlApplet {
    fn name(&self) -> &'static str {
        "nl"
    }
    fn description(&self) -> &'static str {
        "Number lines of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut style = b't';
        let (mut incr, mut num, mut width): (i64, i64, usize) = (1, 1, 6);
        let mut sep = b"\t".to_vec();
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1
                && b[0] == b'-'
                && b != b"-"
                && matches!(b[1], b'b' | b'i' | b'v' | b's' | b'w')
            {
                let v = take_val(b, 1, &mut i, args);
                let s = String::from_utf8_lossy(&v);
                match b[1] {
                    b'b' => {
                        style = *v.first().unwrap_or(&b't');
                        if !matches!(style, b'a' | b't' | b'n') {
                            eprintln!("nl: invalid style");
                            return Ok(1);
                        }
                    }
                    b'i' => {
                        incr = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -i");
                                return Ok(1);
                            }
                        }
                    }
                    b'v' => {
                        num = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -v");
                                return Ok(1);
                            }
                        }
                    }
                    b's' => sep = v,
                    b'w' => {
                        width = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -w");
                                return Ok(1);
                            }
                        }
                    }
                    _ => {}
                }
            } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let mut buf = Vec::with_capacity(4096);
        let mut nbuf = Vec::with_capacity(32);
        for f in &files {
            let mut r = open_input(f)?;
            while next_line(&mut r, &mut buf)? {
                if style == b'a' || (style == b't' && !buf.is_empty()) {
                    nbuf.clear();
                    let mut v = num.unsigned_abs();
                    let mut digs = [0u8; 20];
                    let mut nd = 0;
                    if v == 0 {
                        digs[0] = b'0';
                        nd = 1;
                    }
                    while v > 0 {
                        digs[nd] = (v % 10) as u8 + b'0';
                        v /= 10;
                        nd += 1;
                    }
                    if num < 0 {
                        nbuf.push(b'-');
                    }
                    nbuf.extend(std::iter::repeat_n(
                        b' ',
                        width.saturating_sub(nd + (num < 0) as usize),
                    ));
                    for k in (0..nd).rev() {
                        nbuf.push(digs[k]);
                    }
                    w.write_all(&nbuf)?;
                    w.write_all(&sep)?;
                    num += incr;
                }
                w.write_all(&buf)?;
                w.write_all(b"\n")?;
            }
        }
        Ok(0)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OdType {
    Oct,
    Hex,
    Dec,
    Char,
    Named,
}
fn od_addr(buf: &mut Vec<u8>, off: u64, radix: u8) {
    let mut digs = [0u8; 24];
    let mut nd = 0;
    let mut v = off;
    if v == 0 {
        digs[0] = b'0';
        nd = 1;
    }
    while v > 0 {
        let d = (v % radix as u64) as u8;
        digs[nd] = if d < 10 { d + b'0' } else { d - 10 + b'a' };
        v /= radix as u64;
        nd += 1;
    }
    for _ in nd..7 {
        buf.push(b'0');
    }
    for k in (0..nd).rev() {
        buf.push(digs[k]);
    }
}
fn od_byte(buf: &mut Vec<u8>, b: u8, t: OdType, signed: bool) {
    buf.push(b' ');
    match t {
        OdType::Oct => {
            buf.push(b'0' + (b >> 6));
            buf.push(b'0' + ((b >> 3) & 7));
            buf.push(b'0' + (b & 7));
        }
        OdType::Hex => {
            for sh in [4u8, 0u8] {
                let d = (b >> sh) & 15;
                buf.push(if d < 10 { d + b'0' } else { d - 10 + b'a' });
            }
        }
        OdType::Dec => {
            let neg = signed && b >= 128;
            let m: u16 = if neg { 256 - b as u16 } else { b as u16 };
            if neg {
                buf.push(b'-');
            }
            buf.push(b'0' + (m / 100) as u8);
            buf.push(b'0' + ((m / 10) % 10) as u8);
            buf.push(b'0' + (m % 10) as u8);
        }
        OdType::Char => match b {
            0 => buf.extend_from_slice(b"  \\0"),
            7 => buf.extend_from_slice(b"  \\a"),
            8 => buf.extend_from_slice(b"  \\b"),
            9 => buf.extend_from_slice(b"  \\t"),
            10 => buf.extend_from_slice(b"  \\n"),
            11 => buf.extend_from_slice(b"  \\v"),
            12 => buf.extend_from_slice(b"  \\f"),
            13 => buf.extend_from_slice(b"  \\r"),
            32..=126 => {
                buf.extend_from_slice(b"   ");
                buf.push(b);
            }
            _ => {
                buf.push(b' ');
                buf.push(b'0' + (b >> 6));
                buf.push(b'0' + ((b >> 3) & 7));
                buf.push(b'0' + (b & 7));
            }
        },
        OdType::Named => {
            const N: [&[u8; 3]; 33] = [
                b"nul", b"soh", b"stx", b"etx", b"eot", b"enq", b"ack", b"bel", b" bs", b" ht",
                b" nl", b" vt", b" ff", b" cr", b" so", b" si", b"dle", b"dc1", b"dc2", b"dc3",
                b"dc4", b"nak", b"syn", b"etb", b"can", b" em", b"sub", b"esc", b" fs", b" gs",
                b" rs", b" us", b" sp",
            ];
            if b < 33 {
                buf.push(b' ');
                buf.extend_from_slice(&N[b as usize][..]);
            } else if b == 127 {
                buf.extend_from_slice(b" del");
            } else if b < 127 {
                buf.extend_from_slice(b"   ");
                buf.push(b);
            } else {
                buf.push(b' ');
                buf.push(b'0' + (b >> 6));
                buf.push(b'0' + ((b >> 3) & 7));
                buf.push(b'0' + (b & 7));
            }
        }
    }
}
pub struct OdApplet;
impl Applet for OdApplet {
    fn name(&self) -> &'static str {
        "od"
    }
    fn description(&self) -> &'static str {
        "Dump files in octal and other formats"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (mut radix, mut show) = (8u8, true);
        let (mut typ, mut signed, mut verbose) = (OdType::Oct, false, false);
        let (mut skip, mut max) = (0u64, u64::MAX);
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1 && b[0] == b'-' && b != b"-" {
                let mut j = 1;
                let mut brk = false;
                while j < b.len() {
                    match b[j] {
                        b'v' => {
                            verbose = true;
                            j += 1;
                        }
                        b'x' => {
                            typ = OdType::Hex;
                            signed = false;
                            j += 1;
                        }
                        b'o' | b'B' => {
                            typ = OdType::Oct;
                            signed = false;
                            j += 1;
                        }
                        b'd' => {
                            typ = OdType::Dec;
                            signed = true;
                            j += 1;
                        }
                        b'u' => {
                            typ = OdType::Dec;
                            signed = false;
                            j += 1;
                        }
                        b'c' => {
                            typ = OdType::Char;
                            j += 1;
                        }
                        b'a' => {
                            typ = OdType::Named;
                            j += 1;
                        }
                        b'A' | b't' | b'j' | b'N' => {
                            let opt = b[j];
                            let v = take_val(b, j, &mut i, args);
                            match opt {
                                b'A' => match v.first().copied().unwrap_or(0) {
                                    b'd' => {
                                        radix = 10;
                                        show = true;
                                    }
                                    b'o' => {
                                        radix = 8;
                                        show = true;
                                    }
                                    b'x' => {
                                        radix = 16;
                                        show = true;
                                    }
                                    b'n' => show = false,
                                    _ => {
                                        eprintln!("od: bad -A");
                                        return Ok(1);
                                    }
                                },
                                b't' => match v.as_slice() {
                                    b"x1" => {
                                        typ = OdType::Hex;
                                        signed = false;
                                    }
                                    b"o1" => {
                                        typ = OdType::Oct;
                                        signed = false;
                                    }
                                    b"d1" => {
                                        typ = OdType::Dec;
                                        signed = true;
                                    }
                                    b"u1" => {
                                        typ = OdType::Dec;
                                        signed = false;
                                    }
                                    b"c" => typ = OdType::Char,
                                    b"a" => typ = OdType::Named,
                                    _ => {
                                        eprintln!("od: bad -t (x1/o1/d1/u1/c/a)");
                                        return Ok(1);
                                    }
                                },
                                b'j' | b'N' => {
                                    match String::from_utf8_lossy(&v).trim().parse::<u64>() {
                                        Ok(n) => {
                                            if opt == b'j' {
                                                skip = n;
                                            } else {
                                                max = n;
                                            }
                                        }
                                        Err(_) => {
                                            eprintln!("od: bad number");
                                            return Ok(1);
                                        }
                                    }
                                }
                                _ => {}
                            }
                            brk = true;
                            break;
                        }
                        _ => j += 1,
                    }
                }
                if brk {
                    i += 1;
                    continue;
                }
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let mut chunk = [0u8; 16];
        let mut row = Vec::with_capacity(128);
        let mut prev = [0u8; 16];
        let (mut plen, mut starred, mut off) = (0usize, false, 0u64);
        'fs: for f in &files {
            let mut r = open_input(f)?;
            while skip > 0 {
                let mut d = [0u8; 8192];
                let n = r.read(&mut d[..(skip as usize).min(8192)])?;
                if n == 0 {
                    break;
                }
                skip -= n as u64;
            }
            loop {
                if max == 0 {
                    break 'fs;
                }
                let want = (16usize).min(max as usize);
                let mut got = 0;
                while got < want {
                    match r.read(&mut chunk[got..want])? {
                        0 => break,
                        n => got += n,
                    }
                }
                if got == 0 {
                    break;
                }
                max -= got as u64;
                if !verbose && got == 16 && plen == 16 && chunk == prev {
                    if !starred {
                        w.write_all(b"*\n")?;
                        starred = true;
                    }
                    off += 16;
                    continue;
                }
                starred = false;
                prev[..got].copy_from_slice(&chunk[..got]);
                plen = got;
                row.clear();
                if show {
                    od_addr(&mut row, off, radix);
                }
                for &b in &chunk[..got] {
                    od_byte(&mut row, b, typ, signed);
                }
                row.push(b'\n');
                w.write_all(&row)?;
                off += got as u64;
                if got < 16 {
                    break;
                }
            }
        }
        if show {
            row.clear();
            od_addr(&mut row, off, radix);
            row.push(b'\n');
            w.write_all(&row)?;
        }
        Ok(0)
    }
}

fn convert_stream<R: BufRead, W: Write>(mut r: R, mut w: W, to_unix: bool) -> io::Result<()> {
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
                if pend {
                    pend = false;
                    if b == b'\n' {
                        out.push(b'\n');
                    } else {
                        out.push(b'\r');
                        if b == b'\r' {
                            pend = true;
                        } else {
                            out.push(b);
                        }
                    }
                } else if b == b'\r' {
                    pend = true;
                } else {
                    out.push(b);
                }
            } else {
                if b == b'\n' && !pcr {
                    out.push(b'\r');
                }
                out.push(b);
                pcr = b == b'\r';
            }
        }
        w.write_all(&out)?;
    }
    if to_unix && pend {
        w.write_all(b"\r")?;
    }
    Ok(())
}
fn convert_file(path: &Path, to_unix: bool, tag: &str) -> Result<i32> {
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
    if let Err(e) = r {
        let _ = std::fs::remove_file(&tmp);
        eprintln!("{tag}: {}: {e}", path.display());
        return Ok(1);
    }
    Ok(0)
}
pub struct Dos2unixApplet;
impl Applet for Dos2unixApplet {
    fn name(&self) -> &'static str {
        "dos2unix"
    }
    fn description(&self) -> &'static str {
        "Convert CRLF line endings to LF"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), true)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in &files {
            if *f == Path::new("-") {
                convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), true)?;
            } else if convert_file(f, true, "dos2unix")? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
pub struct Unix2dosApplet;
impl Applet for Unix2dosApplet {
    fn name(&self) -> &'static str {
        "unix2dos"
    }
    fn description(&self) -> &'static str {
        "Convert LF line endings to CRLF"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&Path> = args.iter().map(Path::new).collect();
        if files.is_empty() {
            convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), false)?;
            return Ok(0);
        }
        let mut rc = 0;
        for f in &files {
            if *f == Path::new("-") {
                convert_stream(BufReader::new(io::stdin()), io::stdout().lock(), false)?;
            } else if convert_file(f, false, "unix2dos")? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
