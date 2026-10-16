use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{PathBuf};

struct CompiledRegex {
    preg: libc::regex_t,
}

impl CompiledRegex {
    fn compile(pat: &[u8], extended: bool, icase: bool) -> Option<Self> {
        let c_pat = CString::new(pat).ok()?;
        let mut preg: libc::regex_t = unsafe { std::mem::zeroed() };
        let mut flags = 0;
        if extended {
            flags |= libc::REG_EXTENDED;
        }
        if icase {
            flags |= libc::REG_ICASE;
        }
        if unsafe { libc::regcomp(&mut preg, c_pat.as_ptr(), flags) } != 0 {
            None
        } else {
            Some(Self { preg })
        }
    }

    fn exec(&self, text: &[u8], offset: usize) -> Option<Vec<(usize, usize)>> {
        if offset > text.len() {
            return None;
        }
        let slice = &text[offset..];
        if let Ok(c_slice) = CString::new(slice) {
            let mut pmatch: [libc::regmatch_t; 10] = unsafe { std::mem::zeroed() };
            let eflags = if offset > 0 { libc::REG_NOTBOL } else { 0 };
            let ret = unsafe {
                libc::regexec(&self.preg, c_slice.as_ptr(), 10, pmatch.as_mut_ptr(), eflags)
            };
            if ret == 0 {
                let mut matches = Vec::new();
                for m in &pmatch {
                    if m.rm_so >= 0 && m.rm_eo >= m.rm_so {
                        matches.push((offset + m.rm_so as usize, offset + m.rm_eo as usize));
                    } else {
                        matches.push((0, 0));
                    }
                }
                return Some(matches);
            } else {
                return None;
            }
        }

        // If slice contains embedded NULs, search segment by segment
        let mut cur = 0;
        while cur <= slice.len() {
            let end = slice[cur..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| cur + p)
                .unwrap_or(slice.len());
            let chunk = &slice[cur..end];
            if let Ok(c_chunk) = CString::new(chunk) {
                let mut pmatch: [libc::regmatch_t; 10] = unsafe { std::mem::zeroed() };
                let eflags = if offset + cur > 0 { libc::REG_NOTBOL } else { 0 };
                let ret = unsafe {
                    libc::regexec(&self.preg, c_chunk.as_ptr(), 10, pmatch.as_mut_ptr(), eflags)
                };
                if ret == 0 {
                    let mut matches = Vec::new();
                    for m in &pmatch {
                        if m.rm_so >= 0 && m.rm_eo >= m.rm_so {
                            matches.push((offset + cur + m.rm_so as usize, offset + cur + m.rm_eo as usize));
                        } else {
                            matches.push((0, 0));
                        }
                    }
                    return Some(matches);
                }
            }
            if end >= slice.len() {
                break;
            }
            cur = end + 1;
        }
        None
    }
}

impl Drop for CompiledRegex {
    fn drop(&mut self) {
        unsafe {
            libc::regfree(&mut self.preg);
        }
    }
}

#[derive(Clone, Debug)]
enum AddrSpec {
    None,
    Line(usize),
    Last,
    Pattern(Vec<u8>),
}

#[derive(Clone, Debug)]
enum EndAddrSpec {
    None,
    Line(usize),
    Last,
    Pattern(Vec<u8>),
    Plus(usize),
}

#[derive(Clone, Debug)]
struct Address {
    beg: AddrSpec,
    end: EndAddrSpec,
    invert: bool,
    in_match: bool,
    beg_matched: bool,
    start_line: usize,
}

impl Address {
    fn new(beg: AddrSpec, end: EndAddrSpec, invert: bool) -> Self {
        Self {
            beg,
            end,
            invert,
            in_match: false,
            beg_matched: false,
            start_line: 0,
        }
    }

    fn reset_state(&mut self) {
        self.in_match = false;
        self.beg_matched = false;
        self.start_line = 0;
    }

    fn matches(
        &mut self,
        lineno: usize,
        is_last: bool,
        pattern_space: &[u8],
        extended: bool,
        last_regex: &mut Option<Vec<u8>>,
    ) -> bool {
        let mut matched = false;

        let check_beg = |spec: &AddrSpec, last_re: &mut Option<Vec<u8>>| -> bool {
            match spec {
                AddrSpec::None => true,
                AddrSpec::Line(n) => *n <= lineno,
                AddrSpec::Last => is_last,
                AddrSpec::Pattern(p) => {
                    let pat = if p.is_empty() {
                        last_re.clone().unwrap_or_default()
                    } else {
                        *last_re = Some(p.clone());
                        p.clone()
                    };
                    if let Some(re) = CompiledRegex::compile(&pat, extended, false) {
                        re.exec(pattern_space, 0).is_some()
                    } else {
                        false
                    }
                }
            }
        };

        match (&self.beg, &self.end) {
            (AddrSpec::None, EndAddrSpec::None) => {
                matched = true;
            }
            (beg, EndAddrSpec::None) => {
                matched = match beg {
                    AddrSpec::Line(n) => *n == lineno,
                    _ => check_beg(beg, last_regex),
                };
            }
            (beg, end) => {
                if !self.in_match {
                    if !self.beg_matched && check_beg(beg, last_regex) {
                        matched = true;
                        self.in_match = true;
                        if let AddrSpec::Line(_) = beg {
                            self.beg_matched = true;
                        }
                        self.start_line = lineno;
                        match end {
                            EndAddrSpec::Line(n) if *n <= lineno => {
                                self.in_match = false;
                            }
                            EndAddrSpec::Plus(0) => {
                                self.in_match = false;
                            }
                            _ => {}
                        }
                    }
                } else {
                    matched = true;
                    let end_matched = match end {
                        EndAddrSpec::Line(n) => lineno >= *n,
                        EndAddrSpec::Last => is_last,
                        EndAddrSpec::Plus(n) => lineno >= self.start_line + *n,
                        EndAddrSpec::Pattern(p) => {
                            let pat = if p.is_empty() {
                                last_regex.clone().unwrap_or_default()
                            } else {
                                *last_regex = Some(p.clone());
                                p.clone()
                            };
                            if let Some(re) = CompiledRegex::compile(&pat, extended, false) {
                                re.exec(pattern_space, 0).is_some()
                            } else {
                                false
                            }
                        }
                        EndAddrSpec::None => true,
                    };
                    if end_matched {
                        self.in_match = false;
                    }
                }
            }
        }

        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

#[derive(Clone, Debug)]
enum SedCmd {
    Subst {
        pattern: Vec<u8>,
        replacement: Vec<u8>,
        global: bool,
        which_match: usize,
        print: bool,
        write_file: Option<PathBuf>,
        icase: bool,
    },
    Delete,
    DeleteFirst,
    Print,
    PrintFirst,
    Quit(i32),
    Append(Vec<u8>),
    Insert(Vec<u8>),
    Change(Vec<u8>),
    HoldCopy,
    HoldAppend,
    PatternCopy,
    PatternAppend,
    Exchange,
    Next,
    NextAppend,
    Branch(Option<String>),
    Test(Option<String>, bool),
    Write(PathBuf),
    LineNum,
    Label(String),
    BeginBlock,
    EndBlock,
}

#[derive(Clone, Debug)]
struct CommandRule {
    addr: Address,
    cmd: SedCmd,
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
        let mut in_place_backup: Option<OsString> = None;
        let mut extended = false;
        let mut script_chunks: Vec<Vec<u8>> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();

        let mut i = 0;
        let mut need_script = true;

        while i < args.len() {
            let arg = &args[i];
            let b = arg.as_bytes();
            if b == b"--version" {
                println!("GNU sed version 4.1.5");
                return Ok(0);
            }
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(PathBuf::from(a));
                }
                break;
            }
            if b == b"-" {
                files.push(PathBuf::from("-"));
                i += 1;
                continue;
            }
            if b.starts_with(b"-") && b.len() > 1 {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b'n' => {
                            quiet = true;
                            j += 1;
                        }
                        b'r' | b'E' => {
                            extended = true;
                            j += 1;
                        }
                        b'i' => {
                            in_place = true;
                            if j + 1 < b.len() {
                                in_place_backup = Some(OsString::from(
                                    std::str::from_utf8(&b[j + 1..]).unwrap_or(""),
                                ));
                                break;
                            } else {
                                j += 1;
                            }
                        }
                        b'e' => {
                            let s = if j + 1 < b.len() {
                                b[j + 1..].to_vec()
                            } else if i + 1 < args.len() {
                                i += 1;
                                args[i].as_bytes().to_vec()
                            } else {
                                Vec::new()
                            };
                            script_chunks.push(s);
                            need_script = false;
                            break;
                        }
                        b'f' => {
                            let p = if j + 1 < b.len() {
                                PathBuf::from(std::ffi::OsStr::from_bytes(&b[j + 1..]))
                            } else if i + 1 < args.len() {
                                i += 1;
                                PathBuf::from(&args[i])
                            } else {
                                PathBuf::new()
                            };
                            if let Ok(c) = fs::read(&p) {
                                script_chunks.push(c);
                                need_script = false;
                            }
                            break;
                        }
                        _ => {
                            j += 1;
                        }
                    }
                }
            } else if need_script && script_chunks.is_empty() {
                script_chunks.push(b.to_vec());
                need_script = false;
            } else {
                files.push(PathBuf::from(arg));
            }
            i += 1;
        }

        if in_place && files.is_empty() {
            eprintln!("sed: -i requires at least one file");
            return Ok(1);
        }

        if script_chunks.is_empty() {
            eprintln!("sed: no script provided");
            return Ok(1);
        }

        let full_script = script_chunks.join(&b"\n"[..]);
        let rules = match parse_script(&full_script) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("sed: {e}");
                return Ok(1);
            }
        };

        if files.is_empty() {
            files.push(PathBuf::from("-"));
        }

        execute_sed(&files, &rules, quiet, in_place, in_place_backup.as_deref(), extended)
    }
}

fn parse_script(src: &[u8]) -> std::result::Result<Vec<CommandRule>, String> {
    let mut rules = Vec::new();
    let mut labels = HashMap::new();
    let mut pos = 0;
    parse_commands(src, &mut pos, &mut rules, &mut labels)?;

    // Validate that all branches jump to existing labels
    for r in &rules {
        if let SedCmd::Branch(Some(l)) | SedCmd::Test(Some(l), _) = &r.cmd {
            if !labels.contains_key(l) {
                return Err(format!("undefined label '{l}'"));
            }
        }
    }

    Ok(rules)
}

fn skip_whitespace_and_comments(src: &[u8], pos: &mut usize) {
    while *pos < src.len() {
        let c = src[*pos];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' || c == b';' {
            *pos += 1;
        } else if c == b'#' {
            while *pos < src.len() && src[*pos] != b'\n' {
                *pos += 1;
            }
        } else {
            break;
        }
    }
}

fn parse_commands(
    src: &[u8],
    pos: &mut usize,
    rules: &mut Vec<CommandRule>,
    labels: &mut HashMap<String, usize>,
) -> std::result::Result<(), String> {
    while *pos < src.len() {
        skip_whitespace_and_comments(src, pos);
        if *pos >= src.len() {
            break;
        }

        if src[*pos] == b'}' {
            *pos += 1;
            rules.push(CommandRule {
                addr: Address::new(AddrSpec::None, EndAddrSpec::None, false),
                cmd: SedCmd::EndBlock,
            });
            continue;
        }

        // Parse address 1
        let (addr1, invert) = parse_address(src, pos)?;
        let mut addr2 = EndAddrSpec::None;
        if *pos < src.len() && src[*pos] == b',' {
            *pos += 1;
            addr2 = parse_end_address(src, pos)?;
        }
        let mut final_invert = invert;
        if *pos < src.len() && src[*pos] == b'!' {
            final_invert = !final_invert;
            *pos += 1;
        }

        skip_whitespace(src, pos);
        if *pos >= src.len() {
            break;
        }

        let cmd_char = src[*pos];
        *pos += 1;

        match cmd_char {
            b':' => {
                let lbl = read_token(src, pos);
                labels.insert(lbl.clone(), rules.len());
                rules.push(CommandRule {
                    addr: Address::new(AddrSpec::None, EndAddrSpec::None, false),
                    cmd: SedCmd::Label(lbl),
                });
            }
            b'b' => {
                let lbl = read_token(src, pos);
                let opt_lbl = if lbl.is_empty() { None } else { Some(lbl) };
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Branch(opt_lbl),
                });
            }
            b't' => {
                let lbl = read_token(src, pos);
                let opt_lbl = if lbl.is_empty() { None } else { Some(lbl) };
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Test(opt_lbl, true),
                });
            }
            b'T' => {
                let lbl = read_token(src, pos);
                let opt_lbl = if lbl.is_empty() { None } else { Some(lbl) };
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Test(opt_lbl, false),
                });
            }
            b'd' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Delete,
                });
            }
            b'D' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::DeleteFirst,
                });
            }
            b'p' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Print,
                });
            }
            b'P' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::PrintFirst,
                });
            }
            b'q' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Quit(0),
                });
            }
            b'=' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::LineNum,
                });
            }
            b'h' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::HoldCopy,
                });
            }
            b'H' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::HoldAppend,
                });
            }
            b'g' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::PatternCopy,
                });
            }
            b'G' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::PatternAppend,
                });
            }
            b'x' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Exchange,
                });
            }
            b'n' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Next,
                });
            }
            b'N' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::NextAppend,
                });
            }
            b'a' => {
                let text = read_text_arg(src, pos);
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Append(text),
                });
            }
            b'i' => {
                let text = read_text_arg(src, pos);
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Insert(text),
                });
            }
            b'c' => {
                let text = read_text_arg(src, pos);
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Change(text),
                });
            }
            b'w' => {
                let fname = read_token(src, pos);
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Write(PathBuf::from(fname)),
                });
            }
            b'{' => {
                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::BeginBlock,
                });
            }
            b's' => {
                if *pos >= src.len() {
                    return Err("unterminated 's' command".into());
                }
                let delim = src[*pos];
                *pos += 1;
                let pattern = read_delimited(src, pos, delim, true)?;
                let replacement = read_delimited(src, pos, delim, false)?;
                let mut global = false;
                let mut print = false;
                let mut icase = false;
                let mut which_match = 0;
                let mut write_file = None;

                while *pos < src.len() {
                    let c = src[*pos];
                    if c == b'g' {
                        global = true;
                        *pos += 1;
                    } else if c == b'p' {
                        print = true;
                        *pos += 1;
                    } else if c == b'i' || c == b'I' {
                        icase = true;
                        *pos += 1;
                    } else if c.is_ascii_digit() {
                        let mut num = 0;
                        while *pos < src.len() && src[*pos].is_ascii_digit() {
                            num = num * 10 + (src[*pos] - b'0') as usize;
                            *pos += 1;
                        }
                        which_match = num;
                    } else if c == b'w' {
                        *pos += 1;
                        let fname = read_token(src, pos);
                        write_file = Some(PathBuf::from(fname));
                    } else if c == b';' || c == b'\n' || c == b' ' || c == b'\t' || c == b'}' {
                        break;
                    } else {
                        *pos += 1;
                    }
                }

                rules.push(CommandRule {
                    addr: Address::new(addr1, addr2, final_invert),
                    cmd: SedCmd::Subst {
                        pattern,
                        replacement,
                        global,
                        which_match,
                        print,
                        write_file,
                        icase,
                    },
                });
            }
            _ => {
                // Ignore unrecognized
            }
        }
    }
    Ok(())
}

fn skip_whitespace(src: &[u8], pos: &mut usize) {
    while *pos < src.len() && (src[*pos] == b' ' || src[*pos] == b'\t') {
        *pos += 1;
    }
}

fn parse_address(src: &[u8], pos: &mut usize) -> std::result::Result<(AddrSpec, bool), String> {
    skip_whitespace(src, pos);
    if *pos >= src.len() {
        return Ok((AddrSpec::None, false));
    }
    if src[*pos] == b'$' {
        *pos += 1;
        let mut inv = false;
        if *pos < src.len() && src[*pos] == b'!' {
            inv = true;
            *pos += 1;
        }
        return Ok((AddrSpec::Last, inv));
    }
    if src[*pos].is_ascii_digit() {
        let mut n = 0;
        while *pos < src.len() && src[*pos].is_ascii_digit() {
            n = n * 10 + (src[*pos] - b'0') as usize;
            *pos += 1;
        }
        let mut inv = false;
        if *pos < src.len() && src[*pos] == b'!' {
            inv = true;
            *pos += 1;
        }
        return Ok((AddrSpec::Line(n), inv));
    }
    if src[*pos] == b'/' || (src[*pos] == b'\\' && *pos + 1 < src.len()) {
        let delim = if src[*pos] == b'\\' {
            *pos += 2;
            src[*pos - 1]
        } else {
            *pos += 1;
            b'/'
        };
        let pat = read_delimited(src, pos, delim, true)?;
        let mut inv = false;
        if *pos < src.len() && src[*pos] == b'!' {
            inv = true;
            *pos += 1;
        }
        return Ok((AddrSpec::Pattern(pat), inv));
    }
    Ok((AddrSpec::None, false))
}

fn parse_end_address(src: &[u8], pos: &mut usize) -> std::result::Result<EndAddrSpec, String> {
    skip_whitespace(src, pos);
    if *pos >= src.len() {
        return Ok(EndAddrSpec::None);
    }
    if src[*pos] == b'$' {
        *pos += 1;
        return Ok(EndAddrSpec::Last);
    }
    if src[*pos] == b'+' {
        *pos += 1;
        let mut n = 0;
        while *pos < src.len() && src[*pos].is_ascii_digit() {
            n = n * 10 + (src[*pos] - b'0') as usize;
            *pos += 1;
        }
        return Ok(EndAddrSpec::Plus(n));
    }
    if src[*pos].is_ascii_digit() {
        let mut n = 0;
        while *pos < src.len() && src[*pos].is_ascii_digit() {
            n = n * 10 + (src[*pos] - b'0') as usize;
            *pos += 1;
        }
        return Ok(EndAddrSpec::Line(n));
    }
    if src[*pos] == b'/' {
        *pos += 1;
        let pat = read_delimited(src, pos, b'/', true)?;
        return Ok(EndAddrSpec::Pattern(pat));
    }
    Ok(EndAddrSpec::None)
}

fn read_delimited(
    src: &[u8],
    pos: &mut usize,
    delim: u8,
    is_regex: bool,
) -> std::result::Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut escaped = false;
    let mut in_bracket = false;

    while *pos < src.len() {
        let c = src[*pos];
        *pos += 1;
        if escaped {
            if c == delim && (!is_regex && c == b'&') {
                out.push(b'\\');
                out.push(c);
            } else if c == delim {
                out.push(delim);
            } else if is_regex && c == b'n' {
                out.push(b'\n');
            } else if is_regex && c == b't' {
                out.push(b'\t');
            } else if is_regex && c == b'r' {
                out.push(b'\r');
            } else {
                out.push(b'\\');
                out.push(c);
            }
            escaped = false;
        } else if c == b'\\' {
            escaped = true;
        } else if is_regex && c == b'[' {
            in_bracket = true;
            out.push(c);
            if *pos < src.len() && src[*pos] == b'^' {
                out.push(b'^');
                *pos += 1;
            }
            if *pos < src.len() && src[*pos] == b']' {
                out.push(b']');
                *pos += 1;
            }
        } else if in_bracket && c == b']' {
            in_bracket = false;
            out.push(c);
        } else if in_bracket {
            out.push(c);
        } else if c == delim {
            return Ok(out);
        } else {
            out.push(c);
        }
    }
    Err("unterminated delimited string".into())
}

fn read_token(src: &[u8], pos: &mut usize) -> String {
    skip_whitespace(src, pos);
    let mut out = Vec::new();
    while *pos < src.len() {
        let c = src[*pos];
        if c == b' ' || c == b'\t' || c == b';' || c == b'\n' || c == b'\r' || c == b'}' {
            break;
        }
        out.push(c);
        *pos += 1;
    }
    String::from_utf8_lossy(&out).trim().to_string()
}

fn read_text_arg(src: &[u8], pos: &mut usize) -> Vec<u8> {
    let mut out = Vec::new();
    while *pos < src.len() && (src[*pos] == b' ' || src[*pos] == b'\t') {
        *pos += 1;
    }
    if *pos < src.len() && src[*pos] == b'\\' {
        *pos += 1;
        if *pos < src.len() && src[*pos] == b'\n' {
            *pos += 1;
        }
    }
    while *pos < src.len() {
        let c = src[*pos];
        if c == b'\n' || c == b';' {
            break;
        }
        if c == b'\\' && *pos + 1 < src.len() {
            let next = src[*pos + 1];
            if next == b'n' {
                out.push(b'\n');
                *pos += 2;
                continue;
            } else if next == b't' {
                out.push(b'\t');
                *pos += 2;
                continue;
            } else if next == b'r' {
                out.push(b'\r');
                *pos += 2;
                continue;
            } else if next == b'\\' {
                out.push(b'\\');
                *pos += 2;
                continue;
            }
        }
        out.push(c);
        *pos += 1;
    }
    out
}

const NO_EOL_CHAR: u8 = 1;
const LAST_IS_NUL: u8 = 2;

struct LineItem {
    bytes: Vec<u8>,
    last_gets_char: u8,
}

fn puts_maybe_newline<W: Write>(
    s: &[u8],
    out: &mut W,
    last_puts_char: &mut u8,
    last_gets_char: u8,
) {
    if *last_puts_char != b'\n' && *last_puts_char != b'\0' {
        let _ = out.write_all(b"\n");
        *last_puts_char = b'\n';
    }
    let _ = out.write_all(s);
    if !s.is_empty() {
        *last_puts_char = b'x';
    }
    if last_gets_char == LAST_IS_NUL {
        let _ = out.write_all(b"\0");
        *last_puts_char = b'x';
    } else if last_gets_char != NO_EOL_CHAR {
        let _ = out.write_all(&[last_gets_char]);
        *last_puts_char = last_gets_char;
    }
}

fn execute_sed(
    files: &[PathBuf],
    rules: &[CommandRule],
    quiet: bool,
    in_place: bool,
    backup_sfx: Option<&std::ffi::OsStr>,
    extended: bool,
) -> Result<i32> {
    let mut exit_code = 0;

    if in_place {
        for path in files {
            if path.as_os_str() == "-" {
                eprintln!("sed: cannot edit stdin in-place");
                exit_code = 1;
                continue;
            }
            let content = match fs::read(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("sed: {}: {e}", path.display());
                    exit_code = 1;
                    continue;
                }
            };
            if let Some(sfx) = backup_sfx {
                let mut bkp = path.as_os_str().to_os_string();
                bkp.push(sfx);
                let _ = fs::write(&bkp, &content);
            }
            let mut out = Vec::new();
            let lines = parse_lines_from_bytes(&content);
            let code = run_engine(&lines, rules, quiet, extended, &mut out);
            if code != 0 {
                exit_code = code;
            }
            if let Err(e) = fs::write(path, &out) {
                eprintln!("sed: {}: {e}", path.display());
                exit_code = 1;
            }
        }
    } else {
        let mut all_lines = Vec::new();
        for path in files {
            if path.as_os_str() == "-" {
                let mut buf = Vec::new();
                let _ = io::stdin().lock().read_to_end(&mut buf);
                all_lines.extend(parse_lines_from_bytes(&buf));
            } else {
                match fs::read(path) {
                    Ok(buf) => {
                        all_lines.extend(parse_lines_from_bytes(&buf));
                    }
                    Err(e) => {
                        eprintln!("sed: {}: {e}", path.display());
                        exit_code = 1;
                    }
                }
            }
        }
        let mut stdout = io::stdout().lock();
        let code = run_engine(&all_lines, rules, quiet, extended, &mut stdout);
        if code != 0 {
            exit_code = code;
        }
    }

    Ok(exit_code)
}

fn parse_lines_from_bytes(data: &[u8]) -> Vec<LineItem> {
    let mut out = Vec::new();
    if data.is_empty() {
        return out;
    }
    let mut start = 0;
    while start < data.len() {
        if let Some(pos) = data[start..].iter().position(|&b| b == b'\n') {
            let end = start + pos;
            out.push(LineItem {
                bytes: data[start..end].to_vec(),
                last_gets_char: b'\n',
            });
            start = end + 1;
        } else {
            out.push(LineItem {
                bytes: data[start..].to_vec(),
                last_gets_char: NO_EOL_CHAR,
            });
            break;
        }
    }
    out
}

fn run_engine<W: Write>(
    lines: &[LineItem],
    rules: &[CommandRule],
    quiet: bool,
    extended: bool,
    out: &mut W,
) -> i32 {
    let mut hold_space: Vec<u8> = Vec::new();
    let mut last_regex: Option<Vec<u8>> = None;
    let mut rules = rules.to_vec();
    let mut last_puts_char = b'\n';
    let mut write_files: HashMap<PathBuf, (Option<fs::File>, u8)> = HashMap::new();

    for r in &mut rules {
        r.addr.reset_state();
    }

    let mut labels = HashMap::new();
    for (idx, r) in rules.iter().enumerate() {
        if let SedCmd::Label(l) = &r.cmd {
            labels.insert(l.clone(), idx);
        }
    }

    let mut block_jumps = HashMap::new();
    let mut block_stack = Vec::new();
    for (idx, r) in rules.iter().enumerate() {
        match &r.cmd {
            SedCmd::BeginBlock => {
                block_stack.push(idx);
            }
            SedCmd::EndBlock => {
                if let Some(start_idx) = block_stack.pop() {
                    block_jumps.insert(start_idx, idx);
                }
            }
            _ => {}
        }
    }

    let mut line_idx = 0;
    while line_idx < lines.len() {
        let is_last = line_idx + 1 == lines.len();
        let lineno = line_idx + 1;
        let mut pattern_space = lines[line_idx].bytes.clone();
        let cur_gets_char = lines[line_idx].last_gets_char;
        let mut appends: Vec<Vec<u8>> = Vec::new();
        let mut substituted = false;
        let mut restart_cycle = false;

        let mut cmd_idx = 0;
        while cmd_idx < rules.len() {
            let r = &mut rules[cmd_idx];
            let matched = r.addr.matches(lineno, is_last, &pattern_space, extended, &mut last_regex);

            if !matched {
                if let SedCmd::BeginBlock = &r.cmd {
                    if let Some(&end_idx) = block_jumps.get(&cmd_idx) {
                        cmd_idx = end_idx + 1;
                        continue;
                    }
                }
                cmd_idx += 1;
                continue;
            }

            match &mut r.cmd {
                SedCmd::BeginBlock | SedCmd::EndBlock => {}
                SedCmd::Subst {
                    pattern,
                    replacement,
                    global,
                    which_match,
                    print,
                    write_file,
                    icase,
                } => {
                    let pat = if pattern.is_empty() {
                        last_regex.clone().unwrap_or_default()
                    } else {
                        last_regex = Some(pattern.clone());
                        pattern.clone()
                    };
                    if let Some(re) = CompiledRegex::compile(&pat, extended, *icase) {
                        let (res, did_sub) = do_subst(&pattern_space, &re, replacement, *global, *which_match);
                        if did_sub {
                            pattern_space = res;
                            substituted = true;
                            if *print {
                                puts_maybe_newline(&pattern_space, out, &mut last_puts_char, cur_gets_char);
                            }
                            if let Some(fpath) = write_file {
                                let entry = write_files.entry(fpath.clone()).or_insert_with(|| {
                                    let f = OpenOptions::new().create(true).append(true).open(fpath).ok();
                                    (f, b'\n')
                                });
                                if let (Some(f), last_c) = (&mut entry.0, &mut entry.1) {
                                    puts_maybe_newline(&pattern_space, f, last_c, cur_gets_char);
                                }
                            }
                        }
                    }
                }
                SedCmd::Delete => {
                    restart_cycle = true;
                    break;
                }
                SedCmd::DeleteFirst => {
                    if let Some(pos) = pattern_space.iter().position(|&b| b == b'\n') {
                        pattern_space = pattern_space[pos + 1..].to_vec();
                        cmd_idx = 0;
                        continue;
                    } else {
                        restart_cycle = true;
                        break;
                    }
                }
                SedCmd::Print => {
                    puts_maybe_newline(&pattern_space, out, &mut last_puts_char, cur_gets_char);
                }
                SedCmd::PrintFirst => {
                    let slice = if let Some(pos) = pattern_space.iter().position(|&b| b == b'\n') {
                        &pattern_space[..pos]
                    } else {
                        &pattern_space[..]
                    };
                    puts_maybe_newline(slice, out, &mut last_puts_char, cur_gets_char);
                }
                SedCmd::Quit(code) => {
                    if !quiet {
                        puts_maybe_newline(&pattern_space, out, &mut last_puts_char, cur_gets_char);
                    }
                    return *code;
                }
                SedCmd::Append(text) => {
                    appends.push(text.clone());
                }
                SedCmd::Insert(text) => {
                    puts_maybe_newline(text, out, &mut last_puts_char, b'\n');
                }
                SedCmd::Change(text) => {
                    if !r.addr.in_match {
                        puts_maybe_newline(text, out, &mut last_puts_char, b'\n');
                    }
                    restart_cycle = true;
                    break;
                }
                SedCmd::HoldCopy => {
                    hold_space = pattern_space.clone();
                }
                SedCmd::HoldAppend => {
                    hold_space.push(b'\n');
                    hold_space.extend_from_slice(&pattern_space);
                }
                SedCmd::PatternCopy => {
                    pattern_space = hold_space.clone();
                }
                SedCmd::PatternAppend => {
                    pattern_space.push(b'\n');
                    pattern_space.extend_from_slice(&hold_space);
                }
                SedCmd::Exchange => {
                    std::mem::swap(&mut pattern_space, &mut hold_space);
                }
                SedCmd::Next => {
                    if !quiet {
                        puts_maybe_newline(&pattern_space, out, &mut last_puts_char, cur_gets_char);
                    }
                    line_idx += 1;
                    if line_idx >= lines.len() {
                        return 0;
                    }
                    pattern_space = lines[line_idx].bytes.clone();
                    substituted = false;
                }
                SedCmd::NextAppend => {
                    line_idx += 1;
                    if line_idx >= lines.len() {
                        if !quiet {
                            puts_maybe_newline(&pattern_space, out, &mut last_puts_char, lines[line_idx - 1].last_gets_char);
                        }
                        return 0;
                    }
                    pattern_space.push(b'\n');
                    pattern_space.extend_from_slice(&lines[line_idx].bytes);
                }
                SedCmd::LineNum => {
                    let _ = writeln!(out, "{lineno}");
                    last_puts_char = b'\n';
                }
                SedCmd::Write(fpath) => {
                    let entry = write_files.entry(fpath.clone()).or_insert_with(|| {
                        let f = OpenOptions::new().create(true).append(true).open(fpath).ok();
                        (f, b'\n')
                    });
                    if let (Some(f), last_c) = (&mut entry.0, &mut entry.1) {
                        puts_maybe_newline(&pattern_space, f, last_c, cur_gets_char);
                    }
                }
                SedCmd::Branch(target) => {
                    if let Some(lbl) = target {
                        if let Some(&target_idx) = labels.get(lbl) {
                            cmd_idx = target_idx;
                            continue;
                        }
                    }
                    break;
                }
                SedCmd::Test(target, is_t) => {
                    let should_jump = if *is_t { substituted } else { !substituted };
                    if should_jump {
                        substituted = false;
                        if let Some(lbl) = target {
                            if let Some(&target_idx) = labels.get(lbl) {
                                cmd_idx = target_idx;
                                continue;
                            }
                        }
                        break;
                    }
                }
                SedCmd::Label(_) => {}
            }
            cmd_idx += 1;
        }

        if !restart_cycle {
            if !quiet {
                puts_maybe_newline(&pattern_space, out, &mut last_puts_char, cur_gets_char);
            }
            for app in appends {
                puts_maybe_newline(&app, out, &mut last_puts_char, b'\n');
            }
        }

        line_idx += 1;
    }

    0
}

fn do_subst(
    src: &[u8],
    re: &CompiledRegex,
    repl: &[u8],
    global: bool,
    which_match: usize,
) -> (Vec<u8>, bool) {
    let mut res = Vec::new();
    let mut offset = 0;
    let mut match_count = 0;
    let mut altered = false;
    let mut prev_match_empty = true;
    let mut tried_at_eol = false;

    while offset <= src.len() {
        if let Some(matches) = re.exec(src, offset) {
            let (m_start, m_end) = matches[0];
            let start = m_start - offset;
            let end = m_end - offset;
            match_count += 1;

            if which_match != 0 && which_match != match_count {
                res.extend_from_slice(&src[offset..m_end]);
                offset = m_end;
                if start == end && offset < src.len() {
                    res.push(src[offset]);
                    offset += 1;
                }
                if offset == src.len() {
                    if tried_at_eol {
                        break;
                    }
                    tried_at_eol = true;
                }
                continue;
            }

            res.extend_from_slice(&src[offset..m_start]);

            if prev_match_empty || start != 0 || start != end {
                expand_replacement(&mut res, repl, src, &matches);
                altered = true;
            }

            prev_match_empty = start == end;
            let mut adv = end;
            if prev_match_empty {
                if offset + end >= src.len() {
                    tried_at_eol = true;
                } else {
                    res.push(src[offset + end]);
                    adv += 1;
                }
            }

            offset += adv;

            if !global && which_match == 0 {
                break;
            }
            if which_match != 0 && match_count >= which_match {
                break;
            }

            if offset >= src.len() {
                if tried_at_eol {
                    break;
                }
                tried_at_eol = true;
            }
        } else {
            break;
        }
    }

    if offset < src.len() {
        res.extend_from_slice(&src[offset..]);
    }

    (res, altered)
}

fn expand_replacement(
    out: &mut Vec<u8>,
    repl: &[u8],
    src: &[u8],
    matches: &[(usize, usize)],
) {
    let mut i = 0;
    while i < repl.len() {
        let b = repl[i];
        if b == b'&' {
            let (s, e) = matches[0];
            out.extend_from_slice(&src[s..e]);
            i += 1;
        } else if b == b'\\' && i + 1 < repl.len() {
            let next = repl[i + 1];
            if next.is_ascii_digit() {
                let grp = (next - b'0') as usize;
                if grp < matches.len() {
                    let (s, e) = matches[grp];
                    out.extend_from_slice(&src[s..e]);
                }
                i += 2;
            } else if next == b'n' {
                out.push(b'\n');
                i += 2;
            } else if next == b't' {
                out.push(b'\t');
                i += 2;
            } else if next == b'r' {
                out.push(b'\r');
                i += 2;
            } else if next == b'\\' || next == b'&' {
                out.push(next);
                i += 2;
            } else {
                out.push(next);
                i += 2;
            }
        } else {
            out.push(b);
            i += 1;
        }
    }
}
