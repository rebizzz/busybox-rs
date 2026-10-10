use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;

pub struct AwkApplet;
impl Applet for AwkApplet {
    fn name(&self) -> &'static str {
        "awk"
    }
    fn description(&self) -> &'static str {
        "Pattern scanning and processing language"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fs = " ".to_string();
        let mut prog = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let bytes = args[i].as_bytes();
            if bytes == b"-F" {
                if i + 1 < args.len() {
                    fs = args[i + 1].to_string_lossy().to_string();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-F") {
                fs = args[i].to_string_lossy()[2..].to_string();
            } else if prog.is_none() {
                prog = Some(args[i].to_string_lossy().to_string());
            } else {
                files.push(args[i].clone());
            }
            i += 1;
        }

        let prog_str = match prog {
            Some(p) => p,
            None => {
                eprintln!("awk: program not specified");
                return Ok(1);
            }
        };

        let (begin_blocks, main_rules, end_blocks) = parse_awk_program(&prog_str);

        let mut out = io::stdout().lock();

        for block in &begin_blocks {
            run_awk_action(block, "", &[], &mut out)?;
        }

        let mut process_line = |line: &str| -> io::Result<()> {
            let fields: Vec<String> = if fs == " " {
                line.split_whitespace().map(|s| s.to_string()).collect()
            } else {
                line.split(&fs).map(|s| s.to_string()).collect()
            };

            for (pattern, action) in &main_rules {
                if match_awk_pattern(pattern, line, &fields) {
                    run_awk_action(action, line, &fields, &mut out)?;
                }
            }
            Ok(())
        };

        if files.is_empty() {
            let stdin = io::stdin();
            for l in stdin.lock().lines().map_while(std::result::Result::ok) {
                let _ = process_line(&l);
            }
        } else {
            for f in files {
                if let Ok(file) = File::open(f) {
                    let reader = BufReader::new(file);
                    for l in reader.lines().map_while(std::result::Result::ok) {
                        let _ = process_line(&l);
                    }
                }
            }
        }

        for block in &end_blocks {
            run_awk_action(block, "", &[], &mut out)?;
        }

        let _ = out.flush();
        Ok(0)
    }
}

fn parse_awk_program(prog: &str) -> (Vec<String>, Vec<(String, String)>, Vec<String>) {
    let mut begin = Vec::new();
    let mut rules = Vec::new();
    let mut end = Vec::new();

    let mut rest = prog.trim();
    while !rest.is_empty() {
        if let Some(open) = rest.find('{') {
            let pattern = rest[..open].trim();
            if let Some(close) = rest[open..].find('}') {
                let action = rest[open + 1..open + close].trim().to_string();
                rest = rest[open + close + 1..].trim();

                if pattern == "BEGIN" {
                    begin.push(action);
                } else if pattern == "END" {
                    end.push(action);
                } else {
                    rules.push((pattern.to_string(), action));
                }
            } else {
                break;
            }
        } else {
            rules.push((rest.to_string(), "print $0".into()));
            break;
        }
    }

    if begin.is_empty() && rules.is_empty() && end.is_empty() {
        rules.push(("".into(), "print $0".into()));
    }

    (begin, rules, end)
}

fn match_awk_pattern(pat: &str, line: &str, fields: &[String]) -> bool {
    let pat = pat.trim();
    if pat.is_empty() {
        return true;
    }
    if pat.starts_with('/') && pat.ends_with('/') && pat.len() >= 2 {
        let re = &pat[1..pat.len() - 1];
        return line.contains(re);
    }

    if pat.contains("==") {
        let parts: Vec<&str> = pat.split("==").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let lhs = eval_awk_val(parts[0], line, fields);
            let rhs = eval_awk_val(parts[1], line, fields);
            return lhs == rhs;
        }
    }
    if pat.contains("!=") {
        let parts: Vec<&str> = pat.split("!=").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let lhs = eval_awk_val(parts[0], line, fields);
            let rhs = eval_awk_val(parts[1], line, fields);
            return lhs != rhs;
        }
    }
    line.contains(pat)
}

fn eval_awk_val(v: &str, line: &str, fields: &[String]) -> String {
    let v = v.trim().trim_matches('"');
    if v == "$0" {
        return line.to_string();
    }
    if let Some(idx_str) = v.strip_prefix('$') {
        if let Ok(idx) = idx_str.parse::<usize>() {
            if idx > 0 && idx <= fields.len() {
                return fields[idx - 1].clone();
            }
            return String::new();
        }
    }
    v.to_string()
}

fn run_awk_action(
    action: &str,
    line: &str,
    fields: &[String],
    out: &mut dyn Write,
) -> io::Result<()> {
    for stmt in action.split(';') {
        let stmt = stmt.trim();
        if let Some(expr) = stmt.strip_prefix("print") {
            let expr = expr.trim();
            if expr.is_empty() {
                writeln!(out, "{}", line)?;
            } else {
                let items: Vec<&str> = expr.split(',').map(|s| s.trim()).collect();
                for (i, it) in items.iter().enumerate() {
                    if i > 0 {
                        write!(out, " ")?;
                    }
                    let val = eval_awk_val(it, line, fields);
                    write!(out, "{}", val)?;
                }
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
