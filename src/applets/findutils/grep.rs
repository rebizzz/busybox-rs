use crate::core::{Applet, Result};
use std::collections::VecDeque;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct GrepApplet;
impl Applet for GrepApplet {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "Search for PATTERN in FILEs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "grep",
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}

pub struct EgrepApplet;
impl Applet for EgrepApplet {
    fn name(&self) -> &'static str {
        "egrep"
    }
    fn description(&self) -> &'static str {
        "Alias to grep -E"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "egrep",
            extended_regex: true,
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}

pub struct FgrepApplet;
impl Applet for FgrepApplet {
    fn name(&self) -> &'static str {
        "fgrep"
    }
    fn description(&self) -> &'static str {
        "Alias to grep -F"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let cfg = GrepConfig {
            applet_name: "fgrep",
            fixed_strings: true,
            ..Default::default()
        };
        run_grep(cfg, args)
    }
}

#[derive(Default, Clone)]
struct GrepConfig {
    applet_name: &'static str,
    ignore_case: bool,
    invert_match: bool,
    count_only: bool,
    line_number: bool,
    files_with_matches: bool,
    files_without_matches: bool,
    quiet: bool,
    only_matching: bool,
    extended_regex: bool,
    fixed_strings: bool,
    with_filename: Option<bool>,
    no_messages: bool,
    whole_line: bool,
    whole_word: bool,
    recursive: bool,
    dereference: bool,
    max_matches: Option<usize>,
    before_context: usize,
    after_context: usize,
}

struct PosixRegex {
    preg: libc::regex_t,
}

impl PosixRegex {
    fn compile(pattern: &[u8], extended: bool, icase: bool) -> std::result::Result<Self, String> {
        let c_pattern = CString::new(pattern).map_err(|e| e.to_string())?;
        let mut preg: libc::regex_t = unsafe { std::mem::zeroed() };
        let mut cflags = 0;
        if extended {
            cflags |= libc::REG_EXTENDED;
        }
        if icase {
            cflags |= libc::REG_ICASE;
        }
        let ret = unsafe { libc::regcomp(&mut preg, c_pattern.as_ptr(), cflags) };
        if ret != 0 {
            let mut errbuf = [0u8; 256];
            unsafe {
                libc::regerror(
                    ret,
                    &preg,
                    errbuf.as_mut_ptr() as *mut libc::c_char,
                    errbuf.len(),
                );
            }
            let err_str = unsafe { CStr::from_ptr(errbuf.as_ptr() as *const libc::c_char) }
                .to_string_lossy()
                .into_owned();
            return Err(err_str);
        }
        Ok(PosixRegex { preg })
    }
}

impl Drop for PosixRegex {
    fn drop(&mut self) {
        unsafe {
            libc::regfree(&mut self.preg);
        }
    }
}

enum PatternMatcher {
    Fixed(Vec<u8>),
    Regex(PosixRegex),
}

impl PatternMatcher {
    fn find_match(
        &self,
        line: &[u8],
        ignore_case: bool,
        whole_line: bool,
        whole_word: bool,
    ) -> Option<(usize, usize)> {
        match self {
            PatternMatcher::Fixed(pat) => {
                match_fixed(pat, line, ignore_case, whole_line, whole_word)
            }
            PatternMatcher::Regex(re) => match_regex(re, line, whole_line, whole_word),
        }
    }
}

fn match_fixed(
    pattern: &[u8],
    line: &[u8],
    ignore_case: bool,
    whole_line: bool,
    whole_word: bool,
) -> Option<(usize, usize)> {
    if whole_line {
        let matches = if ignore_case {
            line.eq_ignore_ascii_case(pattern)
        } else {
            line == pattern
        };
        return if matches { Some((0, line.len())) } else { None };
    }

    if pattern.is_empty() {
        if whole_word {
            return None;
        }
        return Some((0, 0));
    }

    if pattern.len() > line.len() {
        return None;
    }

    let mut start = 0;
    while start + pattern.len() <= line.len() {
        let sub = &line[start..start + pattern.len()];
        let matches = if ignore_case {
            sub.eq_ignore_ascii_case(pattern)
        } else {
            sub == pattern
        };
        if matches {
            if whole_word {
                let prev_ok = if start == 0 {
                    true
                } else {
                    let c = line[start - 1];
                    !c.is_ascii_alphanumeric() && c != b'_'
                };
                let next_ok = if start + pattern.len() == line.len() {
                    true
                } else {
                    let c = line[start + pattern.len()];
                    !c.is_ascii_alphanumeric() && c != b'_'
                };
                if prev_ok && next_ok {
                    return Some((start, start + pattern.len()));
                }
                start += 1;
                continue;
            } else {
                return Some((start, start + pattern.len()));
            }
        }
        start += 1;
    }
    None
}

fn match_regex(
    re: &PosixRegex,
    line: &[u8],
    whole_line: bool,
    whole_word: bool,
) -> Option<(usize, usize)> {
    let c_line = match CString::new(line) {
        Ok(c) => c,
        Err(_) => {
            let nul_pos = line.iter().position(|&b| b == 0).unwrap();
            CString::new(&line[..nul_pos]).unwrap()
        }
    };

    let mut pmatch: libc::regmatch_t = unsafe { std::mem::zeroed() };
    let mut offset = 0;
    let mut flags = 0;
    let line_bytes = c_line.to_bytes_with_nul();

    loop {
        if offset >= line_bytes.len() - 1 && offset > 0 {
            return None;
        }
        let ptr = unsafe { line_bytes.as_ptr().add(offset) } as *const libc::c_char;
        let ret = unsafe { libc::regexec(&re.preg, ptr, 1, &mut pmatch, flags) };
        if ret != 0 {
            return None;
        }

        let so = pmatch.rm_so as usize;
        let eo = pmatch.rm_eo as usize;

        if whole_line {
            if offset == 0 && so == 0 && line_bytes[eo] == 0 {
                return Some((0, eo));
            } else {
                return None;
            }
        }

        if !whole_word {
            return Some((offset + so, offset + eo));
        }

        let abs_so = offset + so;
        let abs_eo = offset + eo;

        let prev_ok = if abs_so == 0 {
            true
        } else {
            let c = line_bytes[abs_so - 1];
            !c.is_ascii_alphanumeric() && c != b'_'
        };

        let next_ok = {
            let c = line_bytes[abs_eo];
            if c == 0 {
                true
            } else {
                !c.is_ascii_alphanumeric() && c != b'_'
            }
        };

        if prev_ok && next_ok {
            return Some((abs_so, abs_eo));
        }

        if eo == 0 {
            return None;
        }

        offset += eo;
        flags |= libc::REG_NOTBOL;
    }
}

fn find_all_matches(
    matcher: &PatternMatcher,
    line: &[u8],
    ignore_case: bool,
    whole_line: bool,
    whole_word: bool,
    first_match: (usize, usize),
) -> Vec<(usize, usize)> {
    if whole_line {
        return vec![first_match];
    }
    match matcher {
        PatternMatcher::Fixed(pat) => {
            let mut matches = Vec::new();
            if pat.is_empty() {
                return matches;
            }
            let mut start = 0;
            while start + pat.len() <= line.len() {
                let sub = &line[start..start + pat.len()];
                let eq = if ignore_case {
                    sub.eq_ignore_ascii_case(pat)
                } else {
                    sub == pat
                };
                if eq {
                    if whole_word {
                        let prev_ok = if start == 0 {
                            true
                        } else {
                            let c = line[start - 1];
                            !c.is_ascii_alphanumeric() && c != b'_'
                        };
                        let next_ok = if start + pat.len() == line.len() {
                            true
                        } else {
                            let c = line[start + pat.len()];
                            !c.is_ascii_alphanumeric() && c != b'_'
                        };
                        if prev_ok && next_ok {
                            matches.push((start, start + pat.len()));
                            start += pat.len();
                            continue;
                        }
                        start += 1;
                        continue;
                    } else {
                        matches.push((start, start + pat.len()));
                        start += pat.len();
                        continue;
                    }
                }
                start += 1;
            }
            matches
        }
        PatternMatcher::Regex(re) => {
            let mut result = Vec::new();
            let c_line = match CString::new(line) {
                Ok(c) => c,
                Err(_) => {
                    let nul_pos = line.iter().position(|&b| b == 0).unwrap();
                    CString::new(&line[..nul_pos]).unwrap()
                }
            };
            let line_bytes = c_line.to_bytes_with_nul();

            let (mut start, mut end) = first_match;
            if end > start {
                result.push((start, end));
            }

            loop {
                if end >= line_bytes.len() - 1 {
                    break;
                }
                let search_start = if end == start { end + 1 } else { end };
                if search_start >= line_bytes.len() - 1 {
                    break;
                }

                let ptr = unsafe { line_bytes.as_ptr().add(search_start) } as *const libc::c_char;
                let mut pmatch: libc::regmatch_t = unsafe { std::mem::zeroed() };
                let ret = unsafe { libc::regexec(&re.preg, ptr, 1, &mut pmatch, libc::REG_NOTBOL) };
                if ret != 0 {
                    break;
                }

                let so = search_start + pmatch.rm_so as usize;
                let eo = search_start + pmatch.rm_eo as usize;

                if eo > so {
                    result.push((so, eo));
                }

                start = so;
                end = eo;
            }

            result
        }
    }
}

fn print_line(
    stdout: &mut dyn Write,
    line: &[u8],
    file_display: &str,
    linenum: usize,
    decoration: u8,
    print_filename: bool,
    print_linenum: bool,
) -> io::Result<()> {
    if print_filename {
        stdout.write_all(file_display.as_bytes())?;
        stdout.write_all(&[decoration])?;
    }
    if print_linenum {
        let num_str = linenum.to_string();
        stdout.write_all(num_str.as_bytes())?;
        stdout.write_all(&[decoration])?;
    }
    stdout.write_all(line)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

fn process_dir_entries<F>(dir: &Path, dereference: bool, action: &mut F) -> io::Result<()>
where
    F: FnMut(&Path) -> io::Result<()>,
{
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        entries.push(entry.path());
    }
    entries.sort();

    for path in entries {
        let sym_meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };
        if sym_meta.file_type().is_symlink() {
            if dereference {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.is_dir() {
                        process_dir_entries(&path, dereference, action)?;
                    } else {
                        action(&path)?;
                    }
                }
            } else {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.is_dir() {
                        continue;
                    }
                }
                action(&path)?;
            }
        } else if sym_meta.is_dir() {
            process_dir_entries(&path, dereference, action)?;
        } else {
            action(&path)?;
        }
    }
    Ok(())
}

fn run_grep(mut cfg: GrepConfig, args: &[OsString]) -> Result<i32> {
    let mut pattern_args: Vec<Vec<u8>> = Vec::new();
    let mut pattern_files: Vec<OsString> = Vec::new();
    let mut positional: Vec<OsString> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let bytes = arg.as_bytes();
        if bytes == b"--" {
            i += 1;
            while i < args.len() {
                positional.push(args[i].clone());
                i += 1;
            }
            break;
        }
        if bytes.starts_with(b"--") {
            let s = arg.to_string_lossy();
            if s == "--quiet" || s == "--silent" {
                cfg.quiet = true;
            }

            i += 1;
            continue;
        }
        if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
            let mut j = 1;

            if bytes[1..].iter().all(|b| b.is_ascii_digit()) {
                if let Ok(s) = std::str::from_utf8(&bytes[1..]) {
                    if let Ok(n) = s.parse::<usize>() {
                        cfg.before_context = n;
                        cfg.after_context = n;
                        i += 1;
                        continue;
                    }
                }
            }
            while j < bytes.len() {
                match bytes[j] {
                    b'i' => cfg.ignore_case = true,
                    b'v' => cfg.invert_match = true,
                    b'c' => cfg.count_only = true,
                    b'n' => cfg.line_number = true,
                    b'l' => cfg.files_with_matches = true,
                    b'L' => cfg.files_without_matches = true,
                    b'q' => cfg.quiet = true,
                    b'o' => cfg.only_matching = true,
                    b'E' => cfg.extended_regex = true,
                    b'F' => cfg.fixed_strings = true,
                    b'H' => cfg.with_filename = Some(true),
                    b'h' => cfg.with_filename = Some(false),
                    b's' => cfg.no_messages = true,
                    b'x' => cfg.whole_line = true,
                    b'w' => cfg.whole_word = true,
                    b'r' => cfg.recursive = true,
                    b'R' => {
                        cfg.recursive = true;
                        cfg.dereference = true;
                    }
                    b'a' | b'I' | b'z' => {}
                    b'e' => {
                        let pat = if j + 1 < bytes.len() {
                            let rem = &bytes[j + 1..];
                            rem.to_vec()
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'e'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].as_bytes().to_vec()
                        };
                        pattern_args.push(pat);
                        break;
                    }
                    b'f' => {
                        let f = if j + 1 < bytes.len() {
                            let rem = &bytes[j + 1..];
                            OsStr::from_bytes(rem).to_os_string()
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'f'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].clone()
                        };
                        pattern_files.push(f);
                        break;
                    }
                    b'm' => {
                        let val = if j + 1 < bytes.len() {
                            &bytes[j + 1..]
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'm'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].as_bytes()
                        };
                        cfg.max_matches =
                            std::str::from_utf8(val).ok().and_then(|s| s.parse().ok());
                        break;
                    }
                    b'A' => {
                        let val = if j + 1 < bytes.len() {
                            &bytes[j + 1..]
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'A'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].as_bytes()
                        };
                        cfg.after_context = std::str::from_utf8(val)
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        break;
                    }
                    b'B' => {
                        let val = if j + 1 < bytes.len() {
                            &bytes[j + 1..]
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'B'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].as_bytes()
                        };
                        cfg.before_context = std::str::from_utf8(val)
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        break;
                    }
                    b'C' => {
                        let val = if j + 1 < bytes.len() {
                            &bytes[j + 1..]
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!(
                                    "{}: option requires an argument -- 'C'",
                                    cfg.applet_name
                                );
                                return Ok(2);
                            }
                            args[i].as_bytes()
                        };
                        let c = std::str::from_utf8(val)
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        cfg.before_context = c;
                        cfg.after_context = c;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
            i += 1;
        } else {
            positional.push(arg.clone());
            i += 1;
        }
    }

    if cfg.count_only || cfg.quiet || cfg.files_with_matches || cfg.files_without_matches {
        cfg.line_number = false;
        cfg.before_context = 0;
        cfg.after_context = 0;
    }

    let mut raw_patterns: Vec<Vec<u8>> = Vec::new();
    let had_pattern_options = !pattern_args.is_empty() || !pattern_files.is_empty();

    for pa in pattern_args {
        for p in pa.split(|&b| b == b'\n') {
            raw_patterns.push(p.to_vec());
        }
    }

    let mut fopt_was_specified = false;
    for pf in pattern_files {
        fopt_was_specified = true;
        let mut reader: Box<dyn BufRead> = if pf == "-" {
            Box::new(BufReader::new(io::stdin()))
        } else {
            match File::open(&pf) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => {
                    if !cfg.no_messages {
                        eprintln!("{}: {}: {}", cfg.applet_name, Path::new(&pf).display(), e);
                    }
                    return Ok(2);
                }
            }
        };

        let mut line_buf = Vec::new();
        while let Ok(n) = reader.read_until(b'\n', &mut line_buf) {
            if n == 0 {
                break;
            }
            if line_buf.last() == Some(&b'\n') {
                line_buf.pop();
            }
            if line_buf.last() == Some(&b'\r') {
                line_buf.pop();
            }
            raw_patterns.push(line_buf.clone());
            line_buf.clear();
        }
    }

    let mut input_files: Vec<PathBuf> = Vec::new();
    if !had_pattern_options {
        if positional.is_empty() {
            eprintln!("Usage: {} [OPTIONS] PATTERN [FILE...]", cfg.applet_name);
            return Ok(2);
        }
        let pos_pat = positional.remove(0);
        for p in pos_pat.as_bytes().split(|&b| b == b'\n') {
            raw_patterns.push(p.to_vec());
        }
    }

    for p in positional {
        input_files.push(PathBuf::from(p));
    }

    let mut invert_search = cfg.invert_match;
    if fopt_was_specified && raw_patterns.is_empty() {
        let dummy = if cfg.whole_line {
            b".*".to_vec()
        } else {
            b"".to_vec()
        };
        raw_patterns.push(dummy);
        invert_search = !invert_search;
    }

    let mut compiled_patterns: Vec<PatternMatcher> = Vec::new();
    for rp in &raw_patterns {
        if cfg.fixed_strings {
            compiled_patterns.push(PatternMatcher::Fixed(rp.clone()));
        } else {
            match PosixRegex::compile(rp, cfg.extended_regex, cfg.ignore_case) {
                Ok(re) => compiled_patterns.push(PatternMatcher::Regex(re)),
                Err(e) => {
                    eprintln!(
                        "{}: bad regex '{}': {}",
                        cfg.applet_name,
                        String::from_utf8_lossy(rp),
                        e
                    );
                    return Ok(2);
                }
            }
        }
    }

    if input_files.is_empty() {
        input_files.push(PathBuf::from("-"));
    }

    let mut open_errors = false;
    let mut any_matched = false;
    let stdout = io::stdout();
    let mut out_handle = stdout.lock();

    let mut flat_files: Vec<(PathBuf, bool)> = Vec::new();

    for file_path in &input_files {
        if file_path.as_os_str() == "-" {
            flat_files.push((file_path.clone(), false));
            continue;
        }

        let is_dir = if cfg.dereference {
            std::fs::metadata(file_path)
                .map(|m| m.is_dir())
                .unwrap_or(false)
        } else {
            std::fs::metadata(file_path)
                .map(|m| m.is_dir())
                .unwrap_or(false)
        };

        if cfg.recursive && is_dir {
            let mut sub_action = |p: &Path| {
                flat_files.push((p.to_path_buf(), true));
                Ok(())
            };
            if let Err(e) = process_dir_entries(file_path, cfg.dereference, &mut sub_action) {
                if !cfg.no_messages {
                    eprintln!("{}: {}: {}", cfg.applet_name, file_path.display(), e);
                }
                open_errors = true;
            }
        } else {
            flat_files.push((file_path.clone(), false));
        }
    }

    let default_print_filename = flat_files.len() > 1;

    for (cur_file, is_from_dir) in flat_files {
        let is_stdin = cur_file.as_os_str() == "-";
        let display_name = if is_stdin {
            "(standard input)".to_string()
        } else {
            cur_file.to_string_lossy().into_owned()
        };

        let print_fn = match cfg.with_filename {
            Some(b) => b,
            None => {
                if is_from_dir {
                    true
                } else {
                    default_print_filename
                }
            }
        };

        let reader: Box<dyn BufRead> = if is_stdin {
            Box::new(BufReader::new(io::stdin()))
        } else {
            match File::open(&cur_file) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => {
                    if !cfg.no_messages {
                        eprintln!("{}: {}: {}", cfg.applet_name, cur_file.display(), e);
                    }
                    open_errors = true;
                    continue;
                }
            }
        };

        let res = grep_one_file(
            reader,
            &display_name,
            &compiled_patterns,
            &cfg,
            invert_search,
            print_fn,
            &mut out_handle,
        );

        match res {
            Ok(FileGrepResult::QuietMatch) => {
                return Ok(0);
            }
            Ok(FileGrepResult::Matched(did_match)) => {
                if did_match {
                    any_matched = true;
                }
            }
            Err(e) => {
                if !cfg.no_messages {
                    eprintln!("{}: {}: {}", cfg.applet_name, display_name, e);
                }
                open_errors = true;
            }
        }
    }

    out_handle.flush().ok();

    if cfg.quiet {
        if open_errors {
            Ok(2)
        } else {
            Ok(1)
        }
    } else if open_errors {
        Ok(2)
    } else if any_matched {
        Ok(0)
    } else {
        Ok(1)
    }
}

enum FileGrepResult {
    QuietMatch,
    Matched(bool),
}

fn grep_one_file(
    mut reader: Box<dyn BufRead>,
    display_name: &str,
    patterns: &[PatternMatcher],
    cfg: &GrepConfig,
    invert_search: bool,
    print_filename: bool,
    stdout: &mut dyn Write,
) -> io::Result<FileGrepResult> {
    let mut linenum = 0;
    let mut nmatches = 0;
    let mut line_buf = Vec::new();

    let mut before_buf: VecDeque<(Vec<u8>, usize)> = VecDeque::new();
    let mut print_n_lines_after = 0;
    let mut did_print_line = false;
    let mut last_line_printed = 0;

    while let Ok(n) = reader.read_until(b'\n', &mut line_buf) {
        if n == 0 {
            break;
        }
        linenum += 1;
        if line_buf.last() == Some(&b'\n') {
            line_buf.pop();
        }

        let mut matched_pattern_idx = None;
        let mut first_match_range = None;

        for (idx, pm) in patterns.iter().enumerate() {
            if let Some(range) =
                pm.find_match(&line_buf, cfg.ignore_case, cfg.whole_line, cfg.whole_word)
            {
                matched_pattern_idx = Some(idx);
                first_match_range = Some(range);
                break;
            }
        }

        let found = matched_pattern_idx.is_some();
        let selected = found ^ invert_search;

        if selected {
            nmatches += 1;

            if cfg.quiet {
                return Ok(FileGrepResult::QuietMatch);
            }

            if cfg.files_with_matches {
                writeln!(stdout, "{}", display_name)?;
                return Ok(FileGrepResult::Matched(true));
            }

            if cfg.files_without_matches {
                return Ok(FileGrepResult::Matched(false));
            }

            if !cfg.count_only {
                while let Some((ctx_line, ctx_num)) = before_buf.pop_front() {
                    if (cfg.before_context > 0 || cfg.after_context > 0)
                        && did_print_line
                        && last_line_printed + 1 != ctx_num
                    {
                        writeln!(stdout, "--")?;
                    }
                    print_line(
                        stdout,
                        &ctx_line,
                        display_name,
                        ctx_num,
                        b'-',
                        print_filename,
                        cfg.line_number,
                    )?;
                    did_print_line = true;
                    last_line_printed = ctx_num;
                }

                if (cfg.before_context > 0 || cfg.after_context > 0)
                    && did_print_line
                    && last_line_printed + 1 != linenum
                {
                    writeln!(stdout, "--")?;
                }

                if cfg.only_matching {
                    if !cfg.invert_match {
                        if let (Some(pat_idx), Some(range)) =
                            (matched_pattern_idx, first_match_range)
                        {
                            let all_m = find_all_matches(
                                &patterns[pat_idx],
                                &line_buf,
                                cfg.ignore_case,
                                cfg.whole_line,
                                cfg.whole_word,
                                range,
                            );
                            for (s, e) in all_m {
                                print_line(
                                    stdout,
                                    &line_buf[s..e],
                                    display_name,
                                    linenum,
                                    b':',
                                    print_filename,
                                    cfg.line_number,
                                )?;
                            }
                        }
                    }
                } else {
                    print_line(
                        stdout,
                        &line_buf,
                        display_name,
                        linenum,
                        b':',
                        print_filename,
                        cfg.line_number,
                    )?;
                }

                did_print_line = true;
                last_line_printed = linenum;
                print_n_lines_after = cfg.after_context;
            }
        } else {
            if print_n_lines_after > 0 {
                if (cfg.before_context > 0 || cfg.after_context > 0)
                    && did_print_line
                    && last_line_printed + 1 != linenum
                {
                    writeln!(stdout, "--")?;
                }
                print_line(
                    stdout,
                    &line_buf,
                    display_name,
                    linenum,
                    b'-',
                    print_filename,
                    cfg.line_number,
                )?;
                did_print_line = true;
                last_line_printed = linenum;
                print_n_lines_after -= 1;
            } else if cfg.before_context > 0 {
                if before_buf.len() == cfg.before_context {
                    before_buf.pop_front();
                }
                before_buf.push_back((line_buf.clone(), linenum));
            }
        }

        line_buf.clear();

        if let Some(max_m) = cfg.max_matches {
            if nmatches >= max_m && print_n_lines_after == 0 {
                break;
            }
        }
    }

    if cfg.count_only {
        if print_filename {
            writeln!(stdout, "{}:{}", display_name, nmatches)?;
        } else {
            writeln!(stdout, "{}", nmatches)?;
        }
    }

    if cfg.files_without_matches {
        writeln!(stdout, "{}", display_name)?;
        return Ok(FileGrepResult::Matched(true));
    }

    Ok(FileGrepResult::Matched(nmatches > 0))
}
