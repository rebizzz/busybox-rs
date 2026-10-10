use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn take_arg<'a>(b: &'a [u8], j: usize, i: &mut usize, args: &'a [OsString]) -> Option<&'a [u8]> {
    if j + 1 < b.len() {
        Some(&b[j + 1..])
    } else if *i + 1 < args.len() {
        *i += 1;
        Some(args[*i].as_bytes())
    } else {
        None
    }
}

pub struct SedApplet;

impl Applet for SedApplet {
    fn name(&self) -> &'static str {
        "sed"
    }
    fn description(&self) -> &'static str {
        "Stream editor"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut quiet = false;
        let mut in_place = false;
        let mut scripts: Vec<Vec<u8>> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();

        let mut i = 0;
        let mut script_arg_needed = true;

        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(PathBuf::from(a));
                }
                break;
            }
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b'n' => {
                            quiet = true;
                            j += 1;
                        }
                        b'i' => {
                            in_place = true;
                            j += 1;
                        }
                        b'e' => {
                            if let Some(s) = take_arg(b, j, &mut i, args) {
                                scripts.push(s.to_vec());
                                script_arg_needed = false;
                            }
                            break;
                        }
                        _ => {
                            j += 1;
                        }
                    }
                }
            } else if script_arg_needed && scripts.is_empty() {
                scripts.push(b.to_vec());
                script_arg_needed = false;
            } else {
                files.push(PathBuf::from(&args[i]));
            }
            i += 1;
        }

        if scripts.is_empty() {
            eprintln!("sed: no script provided");
            return Ok(1);
        }

        #[derive(Clone)]
        enum Addr {
            None,
            Line(usize),
            Pattern(Vec<u8>),
            Last,
        }

        #[derive(Clone)]
        enum Action {
            Subst {
                pattern: Vec<u8>,
                replacement: Vec<u8>,
                global: bool,
                print: bool,
            },
            Print,
            Delete,
        }

        #[derive(Clone)]
        struct CommandRule {
            addr: Addr,
            action: Action,
        }

        let mut rules: Vec<CommandRule> = Vec::new();

        for script in &scripts {
            for part in script.split(|&c| c == b'\n' || c == b';') {
                let part = part.trim_ascii();
                if part.is_empty() || part.starts_with(b"#") {
                    continue;
                }

                let mut p = part;
                let mut addr = Addr::None;

                if p.starts_with(b"/") {
                    if let Some(end) = p[1..].iter().position(|&c| c == b'/') {
                        let pat = &p[1..1 + end];
                        addr = Addr::Pattern(pat.to_vec());
                        p = p[end + 2..].trim_ascii();
                    }
                } else if p.starts_with(b"$") {
                    addr = Addr::Last;
                    p = p[1..].trim_ascii();
                } else if !p.is_empty() && p[0].is_ascii_digit() {
                    let mut k = 0;
                    while k < p.len() && p[k].is_ascii_digit() {
                        k += 1;
                    }
                    if let Ok(num) = std::str::from_utf8(&p[..k]).unwrap_or("0").parse::<usize>() {
                        addr = Addr::Line(num);
                    }
                    p = p[k..].trim_ascii();
                }

                if p.is_empty() {
                    continue;
                }

                if p[0] == b'p' {
                    rules.push(CommandRule {
                        addr,
                        action: Action::Print,
                    });
                } else if p[0] == b'd' {
                    rules.push(CommandRule {
                        addr,
                        action: Action::Delete,
                    });
                } else if p[0] == b's' && p.len() >= 3 {
                    let delim = p[1];
                    let rest = &p[2..];
                    let mut fields = Vec::new();
                    let mut cur = Vec::new();
                    let mut escaped = false;
                    let mut rest_idx = 0;

                    for &b in rest {
                        rest_idx += 1;
                        if escaped {
                            cur.push(b);
                            escaped = false;
                        } else if b == b'\\' {
                            escaped = true;
                        } else if b == delim {
                            fields.push(cur.clone());
                            cur.clear();
                            if fields.len() == 2 {
                                break;
                            }
                        } else {
                            cur.push(b);
                        }
                    }

                    if fields.len() == 2 {
                        let flags = &rest[rest_idx..];
                        let global = flags.contains(&b'g');
                        let print = flags.contains(&b'p');
                        rules.push(CommandRule {
                            addr,
                            action: Action::Subst {
                                pattern: fields[0].clone(),
                                replacement: fields[1].clone(),
                                global,
                                print,
                            },
                        });
                    }
                } else if p[0] == b'b' || p[0] == b't' {
                    let label = p[1..].trim_ascii();
                    if !label.is_empty() {
                        eprintln!("sed: undefined label");
                        return Ok(1);
                    }
                }
            }
        }

        if files.is_empty() {
            files.push(PathBuf::from("-"));
        }

        fn sub_bytes(src: &[u8], pat: &[u8], repl: &[u8], global: bool) -> (Vec<u8>, bool) {
            if pat.is_empty() {
                return (src.to_vec(), false);
            }
            let c_pat = match std::ffi::CString::new(pat) {
                Ok(c) => c,
                Err(_) => return (src.to_vec(), false),
            };
            let mut preg: libc::regex_t = unsafe { std::mem::zeroed() };
            if unsafe { libc::regcomp(&mut preg, c_pat.as_ptr(), 0) } != 0 {
                return (src.to_vec(), false);
            }
            let src_nul = match std::ffi::CString::new(src) {
                Ok(c) => c,
                Err(_) => {
                    unsafe {
                        libc::regfree(&mut preg);
                    }
                    return (src.to_vec(), false);
                }
            };
            let mut res = Vec::new();
            let mut offset = 0;
            let mut matched = false;
            let ptr = src_nul.as_ptr();
            loop {
                let mut pmatch: libc::regmatch_t = unsafe { std::mem::zeroed() };
                let flags = if offset > 0 { libc::REG_NOTBOL } else { 0 };
                let ret = unsafe { libc::regexec(&preg, ptr.add(offset), 1, &mut pmatch, flags) };
                if ret == 0 && pmatch.rm_so >= 0 && pmatch.rm_eo >= pmatch.rm_so {
                    matched = true;
                    let m_start = offset + pmatch.rm_so as usize;
                    let m_end = offset + pmatch.rm_eo as usize;
                    res.extend_from_slice(&src[offset..m_start]);
                    for &b in repl {
                        if b == b'&' {
                            res.extend_from_slice(&src[m_start..m_end]);
                        } else {
                            res.push(b);
                        }
                    }
                    if m_start == m_end {
                        if m_end < src.len() {
                            res.push(src[m_end]);
                            offset = m_end + 1;
                        } else {
                            break;
                        }
                    } else {
                        offset = m_end;
                    }
                    if !global || offset >= src.len() {
                        break;
                    }
                } else {
                    break;
                }
            }
            res.extend_from_slice(&src[offset..]);
            unsafe {
                libc::regfree(&mut preg);
            }
            (res, matched)
        }

        fn process_file(
            file: &Path,
            rules: &[CommandRule],
            quiet: bool,
            in_place: bool,
        ) -> Result<()> {
            let (raw_lines, total_lines) = if file == Path::new("-") {
                let stdin = io::stdin();
                let mut lines = Vec::new();
                for l in stdin.lock().lines() {
                    let mut b = l?.into_bytes();
                    b.push(b'\n');
                    lines.push(b);
                }
                let count = lines.len();
                (lines, count)
            } else {
                let content = match fs::read(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("sed: {}: {}", file.display(), e);
                        return Ok(());
                    }
                };
                let mut lines = Vec::new();
                for l in content.split_inclusive(|&b| b == b'\n') {
                    lines.push(l.to_vec());
                }
                let count = lines.len();
                (lines, count)
            };

            let mut out_bytes = Vec::new();

            for (idx, line_with_nl) in raw_lines.iter().enumerate() {
                let line_num = idx + 1;
                let is_last = line_num == total_lines;
                let has_nl = line_with_nl.ends_with(b"\n");
                let line_len = if has_nl {
                    line_with_nl.len() - 1
                } else {
                    line_with_nl.len()
                };
                let mut cur = line_with_nl[..line_len].to_vec();
                let mut deleted = false;
                let mut print_extra = false;

                for rule in rules {
                    let matches_addr = match &rule.addr {
                        Addr::None => true,
                        Addr::Line(n) => *n == line_num,
                        Addr::Pattern(pat) => {
                            if let Ok(c_pat) = std::ffi::CString::new(pat.as_slice()) {
                                let mut preg: libc::regex_t = unsafe { std::mem::zeroed() };
                                if unsafe { libc::regcomp(&mut preg, c_pat.as_ptr(), 0) } == 0 {
                                    if let Ok(c_line) = std::ffi::CString::new(cur.as_slice()) {
                                        let r = unsafe {
                                            libc::regexec(
                                                &preg,
                                                c_line.as_ptr(),
                                                0,
                                                std::ptr::null_mut(),
                                                0,
                                            )
                                        };
                                        unsafe {
                                            libc::regfree(&mut preg);
                                        }
                                        r == 0
                                    } else {
                                        unsafe {
                                            libc::regfree(&mut preg);
                                        }
                                        false
                                    }
                                } else {
                                    cur.windows(pat.len()).any(|w| w == pat.as_slice())
                                }
                            } else {
                                cur.windows(pat.len()).any(|w| w == pat.as_slice())
                            }
                        }
                        Addr::Last => is_last,
                    };

                    if !matches_addr {
                        continue;
                    }

                    match &rule.action {
                        Action::Delete => {
                            deleted = true;
                            break;
                        }
                        Action::Print => {
                            print_extra = true;
                        }
                        Action::Subst {
                            pattern,
                            replacement,
                            global,
                            print,
                        } => {
                            let (new_line, did_sub) =
                                sub_bytes(&cur, pattern, replacement, *global);
                            cur = new_line;
                            if did_sub && *print {
                                print_extra = true;
                            }
                        }
                    }
                }

                if !deleted {
                    if print_extra {
                        out_bytes.extend_from_slice(&cur);
                        out_bytes.push(b'\n');
                    }
                    if !quiet {
                        out_bytes.extend_from_slice(&cur);
                        if has_nl || !is_last {
                            out_bytes.push(b'\n');
                        }
                    }
                }
            }

            if in_place && file != Path::new("-") {
                fs::write(file, out_bytes)?;
            } else {
                io::stdout().lock().write_all(&out_bytes)?;
            }

            Ok(())
        }

        for f in &files {
            process_file(f, &rules, quiet, in_place)?;
        }

        Ok(0)
    }
}

pub struct JoinApplet;

impl Applet for JoinApplet {
    fn name(&self) -> &'static str {
        "join"
    }
    fn description(&self) -> &'static str {
        "Join lines of two files on a common field"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut field1 = 1usize;
        let mut field2 = 1usize;
        let mut delim: Option<u8> = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-1" && i + 1 < args.len() {
                i += 1;
                field1 = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
            } else if b == b"-2" && i + 1 < args.len() {
                i += 1;
                field2 = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
            } else if b == b"-j" && i + 1 < args.len() {
                i += 1;
                let f = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("1")
                    .parse()
                    .unwrap_or(1);
                field1 = f;
                field2 = f;
            } else if b == b"-t" && i + 1 < args.len() {
                i += 1;
                if !args[i].is_empty() {
                    delim = Some(args[i].as_bytes()[0]);
                }
            } else if b.starts_with(b"-t") && b.len() > 2 {
                delim = Some(b[2]);
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(&args[i]);
            }
            i += 1;
        }

        if files.len() != 2 {
            eprintln!("join: requires exactly two files");
            return Ok(1);
        }

        type JoinedRecord = (Vec<u8>, Vec<Vec<u8>>);

        fn read_lines_split(
            path: &OsStr,
            f_idx: usize,
            delim: Option<u8>,
        ) -> Result<Vec<JoinedRecord>> {
            let r: Box<dyn BufRead> = if path == "-" {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(Path::new(path))?))
            };
            let mut out = Vec::new();
            for l in r.lines() {
                let l = l?;
                let b = l.into_bytes();
                let parts: Vec<Vec<u8>> = match delim {
                    Some(d) => b.split(|&c| c == d).map(|s| s.to_vec()).collect(),
                    None => b
                        .split(|&c| c == b' ' || c == b'\t')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_vec())
                        .collect(),
                };
                let key = if f_idx > 0 && f_idx <= parts.len() {
                    parts[f_idx - 1].clone()
                } else {
                    Vec::new()
                };
                out.push((key, parts));
            }
            Ok(out)
        }

        let l1 = match read_lines_split(files[0], field1, delim) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("join: {}", e);
                return Ok(1);
            }
        };
        let l2 = match read_lines_split(files[1], field2, delim) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("join: {}", e);
                return Ok(1);
            }
        };

        let sep = delim.unwrap_or(b' ');
        let out = io::stdout();
        let mut lock = out.lock();

        for (k1, p1) in &l1 {
            for (k2, p2) in &l2 {
                if k1 == k2 && !k1.is_empty() {
                    lock.write_all(k1)?;
                    for (idx, p) in p1.iter().enumerate() {
                        if idx + 1 != field1 {
                            lock.write_all(&[sep])?;
                            lock.write_all(p)?;
                        }
                    }
                    for (idx, p) in p2.iter().enumerate() {
                        if idx + 1 != field2 {
                            lock.write_all(&[sep])?;
                            lock.write_all(p)?;
                        }
                    }
                    lock.write_all(b"\n")?;
                }
            }
        }

        Ok(0)
    }
}

pub struct ExprApplet;

impl Applet for ExprApplet {
    fn name(&self) -> &'static str {
        "expr"
    }
    fn description(&self) -> &'static str {
        "Evaluate expressions"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("expr: missing operand");
            return Ok(2);
        }

        let tokens: Vec<&[u8]> = args.iter().map(|s| s.as_bytes()).collect();

        fn parse_int(s: &[u8]) -> Option<i64> {
            std::str::from_utf8(s).ok()?.parse::<i64>().ok()
        }

        fn eval(tokens: &[&[u8]]) -> std::result::Result<Vec<u8>, i32> {
            if tokens.is_empty() {
                return Err(2);
            }
            if tokens.len() == 1 {
                return Ok(tokens[0].to_vec());
            }

            let ops_order: &[&[&[u8]]] = &[
                &[b"|"],
                &[b"&"],
                &[b"<", b"<=", b"=", b"!=", b">=", b">"],
                &[b"+", b"-"],
                &[b"*", b"/", b"%"],
            ];

            for ops in ops_order {
                for idx in (1..tokens.len() - 1).rev() {
                    let op = tokens[idx];
                    if ops.contains(&op) {
                        let left = eval(&tokens[..idx])?;
                        let right = eval(&tokens[idx + 1..])?;

                        let left_str = std::str::from_utf8(&left).unwrap_or("");
                        let right_str = std::str::from_utf8(&right).unwrap_or("");
                        let left_num = parse_int(&left);
                        let right_num = parse_int(&right);

                        match op {
                            b"|" => {
                                let is_zero_or_null = left.is_empty() || left == b"0";
                                return if !is_zero_or_null {
                                    Ok(left)
                                } else {
                                    let right_zero_or_null = right.is_empty() || right == b"0";
                                    if !right_zero_or_null {
                                        Ok(right)
                                    } else {
                                        Ok(b"0".to_vec())
                                    }
                                };
                            }
                            b"&" => {
                                let l_null = left.is_empty() || left == b"0";
                                let r_null = right.is_empty() || right == b"0";
                                return if !l_null && !r_null {
                                    Ok(left)
                                } else {
                                    Ok(b"0".to_vec())
                                };
                            }
                            b"=" | b"!=" | b"<" | b"<=" | b">" | b">=" => {
                                let cmp = if let (Some(a), Some(b)) = (left_num, right_num) {
                                    a.cmp(&b)
                                } else {
                                    left_str.cmp(right_str)
                                };
                                let res = match op {
                                    b"=" => cmp.is_eq(),
                                    b"!=" => cmp.is_ne(),
                                    b"<" => cmp.is_lt(),
                                    b"<=" => cmp.is_le(),
                                    b">" => cmp.is_gt(),
                                    b">=" => cmp.is_ge(),
                                    _ => false,
                                };
                                return Ok(if res { b"1".to_vec() } else { b"0".to_vec() });
                            }
                            b"+" | b"-" | b"*" | b"/" | b"%" => {
                                let (a, b) = match (left_num, right_num) {
                                    (Some(a), Some(b)) => (a, b),
                                    _ => return Err(2),
                                };
                                let res = match op {
                                    b"+" => a + b,
                                    b"-" => a - b,
                                    b"*" => a * b,
                                    b"/" => {
                                        if b == 0 {
                                            eprintln!("expr: division by zero");
                                            return Err(2);
                                        }
                                        a / b
                                    }
                                    b"%" => {
                                        if b == 0 {
                                            eprintln!("expr: division by zero");
                                            return Err(2);
                                        }
                                        a % b
                                    }
                                    _ => 0,
                                };
                                return Ok(res.to_string().into_bytes());
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(tokens[0].to_vec())
        }

        match eval(&tokens) {
            Ok(res) => {
                let out = io::stdout();
                let mut lock = out.lock();
                lock.write_all(&res)?;
                lock.write_all(b"\n")?;
                let is_null_or_zero = res.is_empty() || res == b"0";
                Ok(if is_null_or_zero { 1 } else { 0 })
            }
            Err(code) => Ok(code),
        }
    }
}

pub struct DiffApplet;

impl Applet for DiffApplet {
    fn name(&self) -> &'static str {
        "diff"
    }
    fn description(&self) -> &'static str {
        "Compare files line by line"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut unified = false;
        let mut files = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-u" || b.starts_with(b"-U") {
                unified = true;
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.len() != 2 {
            eprintln!("diff: missing 2 file operands");
            return Ok(2);
        }

        fn read_lines(p: &Path) -> Result<Vec<Vec<u8>>> {
            let r: Box<dyn BufRead> = if p == Path::new("-") {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(p)?))
            };
            let mut lines = Vec::new();
            for l in r.lines() {
                lines.push(l?.into_bytes());
            }
            Ok(lines)
        }

        let l1 = match read_lines(files[0]) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("diff: {}: {}", files[0].display(), e);
                return Ok(2);
            }
        };
        let l2 = match read_lines(files[1]) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("diff: {}: {}", files[1].display(), e);
                return Ok(2);
            }
        };

        if l1 == l2 {
            return Ok(0);
        }

        let out = io::stdout();
        let mut lock = out.lock();

        if unified {
            writeln!(lock, "--- {}", files[0].display())?;
            writeln!(lock, "+++ {}", files[1].display())?;
            let r1 = if l1.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l1.len())
            };
            let r2 = if l2.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l2.len())
            };
            writeln!(lock, "@@ -{} +{} @@", r1, r2)?;

            let mut i = 0;
            let mut j = 0;
            while i < l1.len() || j < l2.len() {
                if i < l1.len() && j < l2.len() && l1[i] == l2[j] {
                    write!(lock, " ")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                    j += 1;
                } else if i < l1.len() && (j >= l2.len() || !l2.contains(&l1[i])) {
                    write!(lock, "-")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                } else if j < l2.len() {
                    write!(lock, "+")?;
                    lock.write_all(&l2[j])?;
                    writeln!(lock)?;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        } else {
            let mut i = 0;
            let mut j = 0;
            while i < l1.len() || j < l2.len() {
                if i < l1.len() && j < l2.len() && l1[i] == l2[j] {
                    i += 1;
                    j += 1;
                } else if i < l1.len() && (j >= l2.len() || !l2.contains(&l1[i])) {
                    writeln!(lock, "{}d{}", i + 1, j)?;
                    write!(lock, "< ")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                } else if j < l2.len() {
                    writeln!(lock, "{}a{}", i, j + 1)?;
                    write!(lock, "> ")?;
                    lock.write_all(&l2[j])?;
                    writeln!(lock)?;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }

        Ok(1)
    }
}

pub struct PatchApplet;

impl Applet for PatchApplet {
    fn name(&self) -> &'static str {
        "patch"
    }
    fn description(&self) -> &'static str {
        "Apply a patch to files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut strip_count = 0usize;
        let mut reverse = false;
        let mut patch_file: Option<PathBuf> = None;
        let mut target_file: Option<PathBuf> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b.starts_with(b"-p") {
                if b.len() > 2 {
                    strip_count = std::str::from_utf8(&b[2..])
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if i + 1 < args.len() {
                    i += 1;
                    strip_count = std::str::from_utf8(args[i].as_bytes())
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                }
            } else if b == b"-R" || b == b"--reverse" {
                reverse = true;
            } else if b == b"-i" {
                if i + 1 < args.len() {
                    i += 1;
                    patch_file = Some(PathBuf::from(&args[i]));
                }
            } else if b.starts_with(b"-") {
            } else if target_file.is_none() {
                target_file = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }

        let patch_data = match &patch_file {
            Some(p) => match fs::read(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("patch: {}: {}", p.display(), e);
                    return Ok(1);
                }
            },
            None => {
                let mut buf = Vec::new();
                io::stdin().read_to_end(&mut buf)?;
                buf
            }
        };

        let mut deduced_file: Option<PathBuf> = None;
        let mut lines = Vec::new();
        for l in patch_data.split(|&b| b == b'\n') {
            lines.push(l);
        }

        let mut hunk_lines: Vec<&[u8]> = Vec::new();

        for l in &lines {
            if l.starts_with(b"--- ") {
                let name = l[4..].split(|&c| c == b'\t' || c == b' ').next().unwrap();
                if deduced_file.is_none() {
                    let mut parts: Vec<&[u8]> = name.split(|&c| c == b'/').collect();
                    if strip_count < parts.len() {
                        parts = parts[strip_count..].to_vec();
                    }
                    let stripped = parts.join(&b'/');
                    deduced_file = Some(PathBuf::from(OsStr::from_bytes(&stripped)));
                }
            } else if l.starts_with(b"+++ ") {
                let name = l[4..].split(|&c| c == b'\t' || c == b' ').next().unwrap();
                let mut parts: Vec<&[u8]> = name.split(|&c| c == b'/').collect();
                if strip_count < parts.len() {
                    parts = parts[strip_count..].to_vec();
                }
                let stripped = parts.join(&b'/');
                deduced_file = Some(PathBuf::from(OsStr::from_bytes(&stripped)));
            } else if l.starts_with(b"@@ ") {
            } else if l.starts_with(b"+")
                || l.starts_with(b"-")
                || l.starts_with(b" ")
                || l.is_empty()
            {
                hunk_lines.push(l);
            }
        }

        let file_to_patch = target_file.or(deduced_file);
        let path = match file_to_patch {
            Some(p) => p,
            None => {
                eprintln!("patch: cannot determine file to patch");
                return Ok(1);
            }
        };

        let orig_content = fs::read(&path).unwrap_or_default();
        let mut orig_lines: Vec<Vec<u8>> = orig_content
            .split(|&b| b == b'\n')
            .map(|s| s.to_vec())
            .collect();
        if orig_lines.last() == Some(&Vec::new()) {
            orig_lines.pop();
        }

        let mut new_lines = Vec::new();
        let mut orig_idx = 0;

        for hl in hunk_lines {
            if hl.is_empty() {
                continue;
            }
            let tag = hl[0];
            let content = &hl[1..];
            let effective_tag = if reverse {
                if tag == b'+' {
                    b'-'
                } else if tag == b'-' {
                    b'+'
                } else {
                    tag
                }
            } else {
                tag
            };

            match effective_tag {
                b' ' => {
                    new_lines.push(content.to_vec());
                    orig_idx += 1;
                }
                b'+' => {
                    new_lines.push(content.to_vec());
                }
                b'-' => {
                    orig_idx += 1;
                }
                _ => {}
            }
        }

        while orig_idx < orig_lines.len() {
            new_lines.push(orig_lines[orig_idx].clone());
            orig_idx += 1;
        }

        let mut out_data = Vec::new();
        for l in new_lines {
            out_data.extend_from_slice(&l);
            out_data.push(b'\n');
        }

        fs::write(&path, out_data)?;
        println!("patching file {}", path.display());

        Ok(0)
    }
}

pub struct SplitApplet;

impl Applet for SplitApplet {
    fn name(&self) -> &'static str {
        "split"
    }
    fn description(&self) -> &'static str {
        "Split a file into pieces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lines_per_file: Option<usize> = None;
        let mut bytes_per_file: Option<usize> = None;
        let mut numeric_suffix = false;
        let mut file_arg: Option<&Path> = None;
        let mut prefix = b"x".to_vec();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                numeric_suffix = true;
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                lines_per_file = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-l") && b.len() > 2 {
                lines_per_file = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b == b"-b" && i + 1 < args.len() {
                i += 1;
                bytes_per_file = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-b") && b.len() > 2 {
                bytes_per_file = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-") && b.len() > 1 && b[1].is_ascii_digit() {
                lines_per_file = std::str::from_utf8(&b[1..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-") && b != b"-" {
            } else if file_arg.is_none() {
                file_arg = Some(Path::new(&args[i]));
            } else {
                prefix = b.to_vec();
            }
            i += 1;
        }

        let l_limit = lines_per_file.unwrap_or(if bytes_per_file.is_none() { 1000 } else { 0 });
        let input_path = file_arg.unwrap_or_else(|| Path::new("-"));

        let mut reader: Box<dyn Read> = if input_path == Path::new("-") {
            Box::new(io::stdin())
        } else {
            Box::new(open_or_stdin(input_path)?)
        };

        fn make_suffix(idx: usize, numeric: bool) -> Vec<u8> {
            if numeric {
                format!("{:02}", idx).into_bytes()
            } else {
                let first = (b'a' + (idx / 26) as u8) as char;
                let second = (b'a' + (idx % 26) as u8) as char;
                format!("{}{}", first, second).into_bytes()
            }
        }

        if let Some(b_limit) = bytes_per_file {
            let mut file_idx = 0;
            let mut buf = vec![0u8; b_limit];
            loop {
                let mut total_read = 0;
                while total_read < b_limit {
                    let n = reader.read(&mut buf[total_read..])?;
                    if n == 0 {
                        break;
                    }
                    total_read += n;
                }
                if total_read == 0 {
                    break;
                }
                let mut fname = prefix.clone();
                fname.extend_from_slice(&make_suffix(file_idx, numeric_suffix));
                fs::write(OsStr::from_bytes(&fname), &buf[..total_read])?;
                file_idx += 1;
            }
        } else {
            let mut buf_reader = BufReader::new(reader);
            let mut file_idx = 0;
            let mut line = Vec::new();
            let mut done = false;

            while !done {
                let mut cur_file_lines = 0;
                let mut fname = prefix.clone();
                fname.extend_from_slice(&make_suffix(file_idx, numeric_suffix));
                let mut cur_out: Option<File> = None;

                while cur_file_lines < l_limit {
                    line.clear();
                    let n = buf_reader.read_until(b'\n', &mut line)?;
                    if n == 0 {
                        done = true;
                        break;
                    }
                    if cur_out.is_none() {
                        cur_out = Some(File::create(OsStr::from_bytes(&fname))?);
                    }
                    if let Some(ref mut f) = cur_out {
                        f.write_all(&line)?;
                    }
                    cur_file_lines += 1;
                }
                if cur_out.is_some() {
                    file_idx += 1;
                }
            }
        }

        Ok(0)
    }
}

pub struct TacApplet;

impl Applet for TacApplet {
    fn name(&self) -> &'static str {
        "tac"
    }
    fn description(&self) -> &'static str {
        "Concatenate and print files in reverse line order"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut files: Vec<PathBuf> = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(PathBuf::from(arg));
            }
        }

        if files.is_empty() {
            files.push(PathBuf::from("-"));
        }

        let out = io::stdout();
        let mut lock = out.lock();

        for f in &files {
            let mut lines = Vec::new();
            if f == Path::new("-") {
                let stdin = io::stdin();
                for l in stdin.lock().lines() {
                    let mut b = l?.into_bytes();
                    b.push(b'\n');
                    lines.push(b);
                }
            } else {
                let data = match fs::read(f) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("tac: {}: {}", f.display(), e);
                        continue;
                    }
                };
                for l in data.split_inclusive(|&b| b == b'\n') {
                    lines.push(l.to_vec());
                }
            }

            for line in lines.iter().rev() {
                lock.write_all(line)?;
            }
        }

        Ok(0)
    }
}

pub struct ShufApplet;

impl Applet for ShufApplet {
    fn name(&self) -> &'static str {
        "shuf"
    }
    fn description(&self) -> &'static str {
        "Generate random permutations"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count: Option<usize> = None;
        let mut echo_mode = false;
        let mut input_range: Option<(i64, i64)> = None;
        let mut out_file: Option<PathBuf> = None;
        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" {
                echo_mode = true;
            } else if b == b"-n" && i + 1 < args.len() {
                i += 1;
                count = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-n") && b.len() > 2 {
                count = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b == b"-o" && i + 1 < args.len() {
                i += 1;
                out_file = Some(PathBuf::from(&args[i]));
            } else if b == b"-i" && i + 1 < args.len() {
                i += 1;
                if let Some((start, end)) = parse_range(args[i].as_bytes()) {
                    input_range = Some((start, end));
                }
            } else if b.starts_with(b"-i") && b.len() > 2 {
                if let Some((start, end)) = parse_range(&b[2..]) {
                    input_range = Some((start, end));
                }
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                positional.push(&args[i]);
            }
            i += 1;
        }

        fn parse_range(b: &[u8]) -> Option<(i64, i64)> {
            let s = std::str::from_utf8(b).ok()?;
            let mut parts = s.split('-');
            let start = parts.next()?.parse().ok()?;
            let end = parts.next()?.parse().ok()?;
            Some((start, end))
        }

        let mut items: Vec<Vec<u8>> = Vec::new();

        if echo_mode {
            for p in positional {
                items.push(p.as_bytes().to_vec());
            }
        } else if let Some((start, end)) = input_range {
            for num in start..=end {
                items.push(num.to_string().into_bytes());
            }
        } else {
            let p = if positional.is_empty() {
                Path::new("-")
            } else {
                Path::new(positional[0])
            };
            let r: Box<dyn BufRead> = if p == Path::new("-") {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(p)?))
            };
            for l in r.lines() {
                items.push(l?.into_bytes());
            }
        }

        let mut rng_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let n = items.len();
        for k in (1..n).rev() {
            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (rng_seed as usize) % (k + 1);
            items.swap(k, j);
        }

        let out_count = count.unwrap_or(items.len()).min(items.len());
        let mut out_data = Vec::new();
        for item in &items[..out_count] {
            out_data.extend_from_slice(item);
            out_data.push(b'\n');
        }

        if let Some(out_p) = out_file {
            fs::write(out_p, out_data)?;
        } else {
            io::stdout().lock().write_all(&out_data)?;
        }

        Ok(0)
    }
}

pub struct TruncateApplet;

impl Applet for TruncateApplet {
    fn name(&self) -> &'static str {
        "truncate"
    }
    fn description(&self) -> &'static str {
        "Shrink or extend the size of each FILE to the specified size"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut size: Option<u64> = None;
        let mut no_create = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" || b == b"--no-create" {
                no_create = true;
            } else if (b == b"-s" || b == b"--size") && i + 1 < args.len() {
                i += 1;
                size = parse_size(args[i].as_bytes());
            } else if b.starts_with(b"-s") && b.len() > 2 {
                size = parse_size(&b[2..]);
            } else if b.starts_with(b"-") {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        fn parse_size(b: &[u8]) -> Option<u64> {
            let s = std::str::from_utf8(b).ok()?;
            let s = s.trim();
            if s.is_empty() {
                return None;
            }
            let (num_part, mult) = if s.ends_with('K') || s.ends_with('k') {
                (&s[..s.len() - 1], 1024u64)
            } else if s.ends_with('M') || s.ends_with('m') {
                (&s[..s.len() - 1], 1024 * 1024u64)
            } else if s.ends_with('G') || s.ends_with('g') {
                (&s[..s.len() - 1], 1024 * 1024 * 1024u64)
            } else {
                (s, 1u64)
            };
            let val = num_part.parse::<u64>().ok()?;
            Some(val * mult)
        }

        let target_size = match size {
            Some(s) => s,
            None => {
                eprintln!("truncate: missing size option");
                return Ok(1);
            }
        };

        let mut ret = 0;
        for f in files {
            if no_create && !f.exists() {
                continue;
            }
            let file = OpenOptions::new().write(true).create(!no_create).open(f);
            match file {
                Ok(file) => {
                    if let Err(e) = file.set_len(target_size) {
                        eprintln!("truncate: {}: {}", f.display(), e);
                        ret = 1;
                    }
                }
                Err(e) => {
                    eprintln!("truncate: {}: {}", f.display(), e);
                    ret = 1;
                }
            }
        }

        Ok(ret)
    }
}

pub struct TsApplet;

impl Applet for TsApplet {
    fn name(&self) -> &'static str {
        "ts"
    }
    fn description(&self) -> &'static str {
        "Timestamp standard input"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut rel = false;
        let mut format_str = b"%b %d %H:%M:%S".to_vec();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-s" {
                rel = true;
            } else if !b.starts_with(b"-") {
                format_str = b.to_vec();
            }
        }

        let start_time = Instant::now();
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut lock = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            if rel {
                let elapsed = start_time.elapsed().as_secs();
                let hrs = elapsed / 3600;
                let mins = (elapsed % 3600) / 60;
                let secs = elapsed % 60;
                write!(lock, "{:02}:{:02}:{:02} ", hrs, mins, secs)?;
            } else {
                unsafe {
                    let mut tv: libc::timeval = std::mem::zeroed();
                    libc::gettimeofday(&mut tv, std::ptr::null_mut());
                    let mut tm: libc::tm = std::mem::zeroed();
                    libc::localtime_r(&tv.tv_sec, &mut tm);
                    let mut buf = [0u8; 128];
                    let cfmt = CString::new(format_str.clone()).unwrap_or_default();
                    let n = libc::strftime(
                        buf.as_mut_ptr() as *mut libc::c_char,
                        buf.len(),
                        cfmt.as_ptr(),
                        &tm,
                    );
                    if n > 0 {
                        lock.write_all(&buf[..n])?;
                        write!(lock, " ")?;
                    }
                }
            }
            lock.write_all(line.as_bytes())?;
            lock.write_all(b"\n")?;
        }

        Ok(0)
    }
}

pub struct TimeoutApplet;

impl Applet for TimeoutApplet {
    fn name(&self) -> &'static str {
        "timeout"
    }
    fn description(&self) -> &'static str {
        "Run a command with a time limit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("timeout: missing operand");
            return Ok(125);
        }

        let mut sig = libc::SIGTERM;
        let mut idx = 0;

        if args[0].as_bytes() == b"-s" && args.len() >= 3 {
            idx += 2;
            let sig_str = args[1].as_bytes();
            if sig_str == b"KILL" || sig_str == b"9" {
                sig = libc::SIGKILL;
            }
        }

        if idx >= args.len() {
            eprintln!("timeout: missing duration");
            return Ok(125);
        }

        let duration_bytes = args[idx].as_bytes();
        idx += 1;

        let duration_secs: u64 = {
            let s = std::str::from_utf8(duration_bytes).unwrap_or("0");
            let (num_str, mult) = if let Some(stripped) = s.strip_suffix('s') {
                (stripped, 1)
            } else if let Some(stripped) = s.strip_suffix('m') {
                (stripped, 60)
            } else if let Some(stripped) = s.strip_suffix('h') {
                (stripped, 3600)
            } else if let Some(stripped) = s.strip_suffix('d') {
                (stripped, 86400)
            } else {
                (s, 1)
            };
            num_str.parse::<u64>().unwrap_or(0) * mult
        };

        if idx >= args.len() {
            eprintln!("timeout: missing command");
            return Ok(125);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        let mut child = match Command::new(cmd).args(cmd_args).spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "timeout: failed to execute {}: {}",
                    Path::new(cmd).display(),
                    e
                );
                return Ok(127);
            }
        };

        let start = Instant::now();
        let timeout_dur = Duration::from_secs(duration_secs);

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Ok(status.code().unwrap_or(128));
                }
                Ok(None) => {
                    if start.elapsed() >= timeout_dur {
                        let pid = child.id() as i32;
                        unsafe {
                            libc::kill(pid, sig);
                        }
                        let _ = child.wait();
                        return Ok(124);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("timeout: error waiting for child: {}", e);
                    return Ok(125);
                }
            }
        }
    }
}

pub struct DateApplet;

fn print_date_usage() {
    eprintln!(
        "BusyBox-RS v{} multi-call binary.\n\nUsage: date [OPTIONS] [+FMT] [[-s] TIME]\n\nDisplay time (using +FMT), or set time",
        env!("CARGO_PKG_VERSION")
    );
}

impl Applet for DateApplet {
    fn name(&self) -> &'static str {
        "date"
    }
    fn description(&self) -> &'static str {
        "Print or set system date and time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut utc = false;
        let mut format_arg: Option<Vec<u8>> = None;
        let mut date_str: Option<Vec<u8>> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-u" || b == b"--utc" || b == b"--universal" {
                utc = true;
            } else if b == b"-R" || b == b"--rfc-2822" || b == b"--rfc-822" {
                format_arg = Some(b"%a, %d %b %Y %H:%M:%S %z".to_vec());
            } else if b == b"-d" && i + 1 < args.len() {
                i += 1;
                date_str = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-d") && b.len() > 2 {
                date_str = Some(b[2..].to_vec());
            } else if b.starts_with(b"+") {
                format_arg = Some(b[1..].to_vec());
            } else {
                print_date_usage();
                return Ok(1);
            }
            i += 1;
        }

        let mut tm: libc::tm = unsafe { std::mem::zeroed() };

        if let Some(ref d) = date_str {
            if d.starts_with(b"@") {
                let time_sec: libc::time_t = std::str::from_utf8(&d[1..])
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                unsafe {
                    if utc {
                        libc::gmtime_r(&time_sec, &mut tm);
                    } else {
                        libc::localtime_r(&time_sec, &mut tm);
                    }
                }
            } else if let Ok(s) = std::str::from_utf8(d) {
                let now: libc::time_t = unsafe { libc::time(std::ptr::null_mut()) };
                let mut base_tm: libc::tm = unsafe { std::mem::zeroed() };
                unsafe {
                    if utc {
                        libc::gmtime_r(&now, &mut base_tm);
                    } else {
                        libc::localtime_r(&now, &mut base_tm);
                    }
                }
                base_tm.tm_hour = 0;
                base_tm.tm_min = 0;
                base_tm.tm_sec = 0;

                let fmts = [
                    "%R",
                    "%T",
                    "%m.%d-%R",
                    "%m.%d-%T",
                    "%Y.%m.%d-%R",
                    "%Y.%m.%d-%T",
                    "%b %d %T %Y",
                    "%Y-%m-%d %R %z",
                    "%Y-%m-%d %T %z",
                    "%Y-%m-%d %R%z",
                    "%Y-%m-%d %T%z",
                    "%Y-%m-%d %R",
                    "%Y-%m-%d %T",
                    "%Y-%m-%d %H",
                    "%Y-%m-%d",
                ];

                let mut parsed = false;
                let mut has_tz = false;
                if let Ok(c_str) = CString::new(s) {
                    for f in &fmts {
                        if let Ok(c_fmt) = CString::new(*f) {
                            let mut test_tm = base_tm;
                            let res = unsafe {
                                libc::strptime(c_str.as_ptr(), c_fmt.as_ptr(), &mut test_tm)
                            };
                            if !res.is_null() && unsafe { *res == 0 } {
                                parsed = true;
                                has_tz = f.contains(&"%z");
                                tm = test_tm;
                                break;
                            }
                        }
                    }
                }

                if !parsed {
                    let dot_pos = s.find('.');
                    let main_part = match dot_pos {
                        Some(p) => &s[..p],
                        None => s,
                    };
                    let sec: i32 = match dot_pos {
                        Some(p) => match s[p + 1..].parse() {
                            Ok(sec_val) => sec_val,
                            Err(_) => {
                                print_date_usage();
                                return Ok(1);
                            }
                        },
                        None => 0,
                    };
                    if main_part.chars().all(|c| c.is_ascii_digit()) && main_part.len() == 12 {
                        let y: i32 = main_part[0..4].parse().unwrap_or(0);
                        let m: i32 = main_part[4..6].parse().unwrap_or(0);
                        let day: i32 = main_part[6..8].parse().unwrap_or(0);
                        let h: i32 = main_part[8..10].parse().unwrap_or(0);
                        let min: i32 = main_part[10..12].parse().unwrap_or(0);
                        tm = base_tm;
                        tm.tm_year = y - 1900;
                        tm.tm_mon = m - 1;
                        tm.tm_mday = day;
                        tm.tm_hour = h;
                        tm.tm_min = min;
                        tm.tm_sec = sec;
                        parsed = true;
                    }
                }

                if !parsed {
                    print_date_usage();
                    return Ok(1);
                }

                unsafe {
                    let t: libc::time_t = if has_tz {
                        tm.tm_sec -= tm.tm_gmtoff as libc::c_int;
                        tm.tm_isdst = 0;
                        libc::timegm(&mut tm)
                    } else if utc {
                        tm.tm_isdst = 0;
                        libc::timegm(&mut tm)
                    } else {
                        tm.tm_isdst = -1;
                        libc::mktime(&mut tm)
                    };
                    if utc {
                        libc::gmtime_r(&t, &mut tm);
                    } else {
                        libc::localtime_r(&t, &mut tm);
                    }
                }
            } else {
                print_date_usage();
                return Ok(1);
            }
        } else {
            let now: libc::time_t = unsafe { libc::time(std::ptr::null_mut()) };
            unsafe {
                if utc {
                    libc::gmtime_r(&now, &mut tm);
                } else {
                    libc::localtime_r(&now, &mut tm);
                }
            }
        }

        unsafe {
            let fmt_str = if let Some(f) = format_arg {
                f
            } else if utc {
                b"%a %b %e %H:%M:%S UTC %Y".to_vec()
            } else {
                b"%a %b %e %H:%M:%S %Z %Y".to_vec()
            };
            let cfmt = CString::new(fmt_str).unwrap_or_default();
            let mut buf = [0u8; 256];
            let len = libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                cfmt.as_ptr(),
                &tm,
            );
            if len > 0 {
                let out = io::stdout();
                let mut lock = out.lock();
                lock.write_all(&buf[..len])?;
                lock.write_all(b"\n")?;
            }
        }

        Ok(0)
    }
}

pub struct EnvApplet;

impl Applet for EnvApplet {
    fn name(&self) -> &'static str {
        "env"
    }
    fn description(&self) -> &'static str {
        "Set environment and run command, or print environment"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ignore_env = false;
        let mut idx = 0;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-i" || b == b"-" || b == b"--ignore-environment" {
                ignore_env = true;
            } else if b.starts_with(b"-") {
            } else {
                break;
            }
            idx += 1;
        }

        let mut custom_env: Vec<(OsString, OsString)> = Vec::new();
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if let Some(pos) = b.iter().position(|&c| c == b'=') {
                let k = OsStr::from_bytes(&b[..pos]).to_os_string();
                let v = OsStr::from_bytes(&b[pos + 1..]).to_os_string();
                custom_env.push((k, v));
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            let out = io::stdout();
            let mut lock = out.lock();
            if !ignore_env {
                for (k, v) in std::env::vars_os() {
                    if !custom_env.iter().any(|(ck, _)| ck == &k) {
                        lock.write_all(k.as_bytes())?;
                        lock.write_all(b"=")?;
                        lock.write_all(v.as_bytes())?;
                        lock.write_all(b"\n")?;
                    }
                }
            }
            for (k, v) in custom_env {
                lock.write_all(k.as_bytes())?;
                lock.write_all(b"=")?;
                lock.write_all(v.as_bytes())?;
                lock.write_all(b"\n")?;
            }
            return Ok(0);
        }

        let prog = &args[idx];
        let prog_args = &args[idx + 1..];

        let mut cmd = Command::new(prog);
        cmd.args(prog_args);
        if ignore_env {
            cmd.env_clear();
        }
        for (k, v) in custom_env {
            cmd.env(k, v);
        }

        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("env: {}: {}", Path::new(prog).display(), e);
                Ok(127)
            }
        }
    }
}

pub struct MktempApplet;

impl Applet for MktempApplet {
    fn name(&self) -> &'static str {
        "mktemp"
    }
    fn description(&self) -> &'static str {
        "Create temporary file or directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut directory = false;
        let mut quiet = false;
        let mut template: Option<Vec<u8>> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-d" || b == b"--directory" {
                directory = true;
            } else if b == b"-q" || b == b"--quiet" {
                quiet = true;
            } else if !b.starts_with(b"-") {
                template = Some(b.to_vec());
            }
        }

        let templ = template.unwrap_or_else(|| b"/tmp/tmp.XXXXXX".to_vec());
        let mut ctempl = templ.clone();
        if !ctempl.ends_with(b"XXXXXX") {
            ctempl.extend_from_slice(b".XXXXXX");
        }
        ctempl.push(0);

        let ptr = ctempl.as_mut_ptr() as *mut libc::c_char;

        let res = unsafe {
            if directory {
                if libc::mkdtemp(ptr).is_null() {
                    -1
                } else {
                    0
                }
            } else {
                let fd = libc::mkstemp(ptr);
                if fd >= 0 {
                    libc::close(fd);
                    0
                } else {
                    -1
                }
            }
        };

        if res < 0 {
            if !quiet {
                eprintln!("mktemp: failed to create temporary file/directory");
            }
            return Ok(1);
        }

        ctempl.pop();
        let out = io::stdout();
        let mut lock = out.lock();
        lock.write_all(&ctempl)?;
        lock.write_all(b"\n")?;

        Ok(0)
    }
}

pub struct InstallApplet;

impl Applet for InstallApplet {
    fn name(&self) -> &'static str {
        "install"
    }
    fn description(&self) -> &'static str {
        "Copy files and set attributes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut directory_mode = false;
        let mut mode: Option<u32> = None;
        let mut targets: Vec<&Path> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" || b == b"--directory" {
                directory_mode = true;
            } else if b == b"-m" || b == b"--mode" {
                if i + 1 < args.len() {
                    i += 1;
                    mode = u32::from_str_radix(
                        std::str::from_utf8(args[i].as_bytes()).unwrap_or("755"),
                        8,
                    )
                    .ok();
                }
            } else if b.starts_with(b"-m") && b.len() > 2 {
                mode = u32::from_str_radix(std::str::from_utf8(&b[2..]).unwrap_or("755"), 8).ok();
            } else if b.starts_with(b"-") {
            } else {
                targets.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if directory_mode {
            let dir_mode = mode.unwrap_or(0o755);
            for d in targets {
                if let Err(e) = fs::create_dir_all(d) {
                    eprintln!("install: {}: {}", d.display(), e);
                    return Ok(1);
                }
                let _ = fs::set_permissions(d, fs::Permissions::from_mode(dir_mode));
            }
            return Ok(0);
        }

        if targets.len() < 2 {
            eprintln!("install: missing destination file operand");
            return Ok(1);
        }

        let file_mode = mode.unwrap_or(0o755);
        let dest = targets.last().unwrap();
        let sources = &targets[..targets.len() - 1];

        if dest.is_dir() {
            for src in sources {
                let fname = match src.file_name() {
                    Some(f) => f,
                    None => continue,
                };
                let target_path = dest.join(fname);
                if let Err(e) = fs::copy(src, &target_path) {
                    eprintln!("install: {}: {}", src.display(), e);
                    return Ok(1);
                }
                let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(file_mode));
            }
        } else {
            if sources.len() > 1 {
                eprintln!("install: target '{}' is not a directory", dest.display());
                return Ok(1);
            }
            if let Err(e) = fs::copy(sources[0], dest) {
                eprintln!("install: {}: {}", sources[0].display(), e);
                return Ok(1);
            }
            let _ = fs::set_permissions(dest, fs::Permissions::from_mode(file_mode));
        }

        Ok(0)
    }
}

pub struct HostidApplet;

impl Applet for HostidApplet {
    fn name(&self) -> &'static str {
        "hostid"
    }
    fn description(&self) -> &'static str {
        "Print the numeric identifier for the current host"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let id = unsafe { libc::gethostid() };
        println!("{:08x}", id as u32);
        Ok(0)
    }
}
