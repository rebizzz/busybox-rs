use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};

pub struct ExpandApplet;
impl Applet for ExpandApplet {
    fn name(&self) -> &'static str { "expand" }
    fn description(&self) -> &'static str { "Convert tabs to spaces" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tab_size: usize = 8;
        let mut opt_initial = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-i" || bytes == b"--initial" {
                opt_initial = true;
            } else if bytes == b"-t" || bytes == b"--tabs" {
                if i + 1 < args.len() {
                    tab_size = args[i + 1].to_string_lossy().parse().unwrap_or(8);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-t") {
                tab_size = arg.to_string_lossy()[2..].parse().unwrap_or(8);
            } else if !bytes.starts_with(b"-") {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;
            let mut ptr_strbeg = 0;
            let mut ptr = 0;

            while ptr < content.len() {
                let c = content[ptr];
                if c == b'\n' {
                    handle.write_all(&content[ptr_strbeg..=ptr])?;
                    ptr += 1;
                    ptr_strbeg = ptr;
                    continue;
                }

                if opt_initial && c != b' ' && c != b'\t' {
                    while ptr < content.len() && content[ptr] != b'\n' {
                        ptr += 1;
                    }
                    continue;
                }

                if c == b'\t' {
                    let chunk = &content[ptr_strbeg..ptr];
                    handle.write_all(chunk)?;
                    let w = unicode_strwidth(chunk);
                    let spaces = tab_size - (w % tab_size);
                    for _ in 0..spaces {
                        handle.write_all(b" ")?;
                    }
                    ptr += 1;
                    ptr_strbeg = ptr;
                    continue;
                }
                ptr += 1;
            }

            if ptr_strbeg < content.len() {
                handle.write_all(&content[ptr_strbeg..])?;
            }
        }
        Ok(0)
    }
}

pub struct UnexpandApplet;
impl Applet for UnexpandApplet {
    fn name(&self) -> &'static str { "unexpand" }
    fn description(&self) -> &'static str { "Convert spaces to tabs" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tab_size: usize = 8;
        let mut opt_all = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-a" || bytes == b"--all" {
                opt_all = true;
            } else if bytes == b"-f" || bytes == b"--first-only" {
                opt_all = false;
            } else if bytes == b"-t" || bytes == b"--tabs" {
                if i + 1 < args.len() {
                    tab_size = args[i + 1].to_string_lossy().parse().unwrap_or(8);
                    opt_all = true;
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-t") {
                tab_size = arg.to_string_lossy()[2..].parse().unwrap_or(8);
                opt_all = true;
            } else if bytes.starts_with(b"-") && bytes.len() > 1 {
                for &b in &bytes[1..] {
                    if b == b'a' { opt_all = true; }
                    else if b == b'f' { opt_all = false; }
                }
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        // In BusyBox: "ta" (-t NUM sets -a), but "if (opt & OPT_INITIAL) opt &= ~OPT_ALL;"
        // -f sets OPT_INITIAL, so if -f is present, it explicitly disables -a!
        let has_f = args.iter().any(|a| {
            let b = a.as_bytes();
            b == b"-f" || b == b"--first-only"
        });
        if has_f {
            opt_all = false;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;

            let mut ptr_line_start = 0;
            while ptr_line_start < content.len() {
                let mut line_end = ptr_line_start;
                while line_end < content.len() && content[line_end] != b'\n' {
                    line_end += 1;
                }
                let has_newline = line_end < content.len() && content[line_end] == b'\n';
                let line = &content[ptr_line_start..line_end];
                ptr_line_start = if has_newline { line_end + 1 } else { line_end };

                let mut ptr = 0;
                let mut column = 0;

                while ptr < line.len() {
                    let mut len = 0;
                    while ptr < line.len() && line[ptr] == b' ' {
                        ptr += 1;
                        len += 1;
                    }
                    column += len;

                    if ptr < line.len() && line[ptr] == b'\t' {
                        column += tab_size - (column % tab_size);
                        ptr += 1;
                        continue;
                    }

                    let n = column / tab_size;
                    if n > 0 {
                        len = column % tab_size;
                        column = len;
                        for _ in 0..n {
                            handle.write_all(b"\t")?;
                        }
                    }

                    if !opt_all && ptr != 0 {
                        for _ in 0..len {
                            handle.write_all(b" ")?;
                        }
                        handle.write_all(&line[ptr..])?;
                        break;
                    }

                    let mut nspan = 0;
                    while ptr + nspan < line.len() && line[ptr + nspan] != b' ' && line[ptr + nspan] != b'\t' {
                        nspan += 1;
                    }

                    for _ in 0..len {
                        handle.write_all(b" ")?;
                    }
                    handle.write_all(&line[ptr..ptr + nspan])?;

                    let char_width = unicode_strwidth(&line[ptr..ptr + nspan]);
                    ptr += nspan;
                    column = (column + char_width) % tab_size;
                }

                if has_newline {
                    handle.write_all(b"\n")?;
                }
            }
        }
        Ok(0)
    }
}

fn unicode_strwidth(s: &[u8]) -> usize {
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
