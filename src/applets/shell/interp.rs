use crate::core::{Applet, Result};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

fn run_command_line(line: &str) -> i32 {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return 0;
    }

    let mut rest = line;
    let mut last_status = 0;

    while !rest.is_empty() {
        let mut next_delim = None;
        let mut idx = rest.len();

        if let Some(i) = rest.find("&&") {
            if i < idx {
                idx = i;
                next_delim = Some("&&");
            }
        }
        if let Some(i) = rest.find("||") {
            if i < idx {
                idx = i;
                next_delim = Some("||");
            }
        }
        if let Some(i) = rest.find(';') {
            if i < idx {
                idx = i;
                next_delim = Some(";");
            }
        }

        let cmd_part = rest[..idx].trim();
        if let Some(delim) = next_delim {
            rest = rest[idx + delim.len()..].trim_start();
        } else {
            rest = "";
        }

        if cmd_part.is_empty() {
            continue;
        }

        last_status = execute_pipeline(cmd_part);

        if let Some(delim) = next_delim {
            if (delim == "&&" && last_status != 0) || (delim == "||" && last_status == 0) {
                break;
            }
        }
    }

    last_status
}

fn execute_pipeline(pipeline: &str) -> i32 {
    let stages: Vec<&str> = pipeline.split('|').map(|s| s.trim()).collect();
    if stages.is_empty() {
        return 0;
    }

    if stages.len() == 1 {
        return execute_single_command(stages[0], Stdio::inherit(), Stdio::inherit());
    }

    let mut children: Vec<Child> = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;

    for (i, stage) in stages.iter().enumerate() {
        let is_last = i == stages.len() - 1;
        let stdin = match prev_stdout.take() {
            Some(out) => Stdio::from(out),
            None => Stdio::inherit(),
        };
        let stdout = if is_last {
            Stdio::inherit()
        } else {
            Stdio::piped()
        };

        let parsed = match parse_simple_command(stage) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("sh: {}", e);
                return 1;
            }
        };

        if parsed.args.is_empty() {
            continue;
        }

        if is_builtin(&parsed.args[0]) {
            let code = run_builtin(&parsed.args);
            if is_last {
                return code;
            }
            continue;
        }

        let mut cmd = Command::new(&parsed.args[0]);
        cmd.args(&parsed.args[1..]);
        cmd.stdin(stdin);
        cmd.stdout(stdout);

        match cmd.spawn() {
            Ok(mut child) => {
                if !is_last {
                    prev_stdout = child.stdout.take();
                }
                children.push(child);
            }
            Err(e) => {
                eprintln!("sh: {}: {}", parsed.args[0], e);
                return 127;
            }
        }
    }

    let mut last_code = 0;
    for mut child in children {
        if let Ok(st) = child.wait() {
            last_code = st.code().unwrap_or(1);
        }
    }

    last_code
}

struct ParsedCommand {
    args: Vec<String>,
    stdin_file: Option<String>,
    stdout_file: Option<String>,
    stdout_append: bool,
}

fn parse_simple_command(s: &str) -> std::result::Result<ParsedCommand, String> {
    let tokens = tokenize(s);
    let mut args = Vec::new();
    let mut stdin_file = None;
    let mut stdout_file = None;
    let mut stdout_append = false;

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        if tok == "<" {
            if i + 1 >= tokens.len() {
                return Err("syntax error near unexpected token '<'".into());
            }
            stdin_file = Some(tokens[i + 1].clone());
            i += 2;
        } else if tok == ">" {
            if i + 1 >= tokens.len() {
                return Err("syntax error near unexpected token '>'".into());
            }
            stdout_file = Some(tokens[i + 1].clone());
            stdout_append = false;
            i += 2;
        } else if tok == ">>" {
            if i + 1 >= tokens.len() {
                return Err("syntax error near unexpected token '>>'".into());
            }
            stdout_file = Some(tokens[i + 1].clone());
            stdout_append = true;
            i += 2;
        } else {
            args.push(tok.clone());
            i += 1;
        }
    }

    Ok(ParsedCommand {
        args,
        stdin_file,
        stdout_file,
        stdout_append,
    })
}

fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_single {
            if c == '\'' {
                in_single = false;
            } else {
                cur.push(c);
            }
        } else if in_double {
            if c == '"' {
                in_double = false;
            } else if c == '\\' && i + 1 < chars.len() {
                i += 1;
                cur.push(chars[i]);
            } else {
                cur.push(c);
            }
        } else {
            match c {
                '\'' => in_single = true,
                '"' => in_double = true,
                '\\' => {
                    if i + 1 < chars.len() {
                        i += 1;
                        cur.push(chars[i]);
                    }
                }
                ' ' | '\t' | '\r' | '\n' => {
                    if !cur.is_empty() {
                        tokens.push(cur);
                        cur = String::new();
                    }
                }
                '>' => {
                    if !cur.is_empty() {
                        tokens.push(cur);
                        cur = String::new();
                    }
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(">>".into());
                        i += 1;
                    } else {
                        tokens.push(">".into());
                    }
                }
                '<' => {
                    if !cur.is_empty() {
                        tokens.push(cur);
                        cur = String::new();
                    }
                    tokens.push("<".into());
                }
                _ => cur.push(c),
            }
        }
        i += 1;
    }

    if !cur.is_empty() {
        tokens.push(cur);
    }

    tokens
}

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "cd" | "exit" | "echo" | "pwd" | "exec" | ":")
}

fn run_builtin(args: &[String]) -> i32 {
    if args.is_empty() {
        return 0;
    }
    match args[0].as_str() {
        "cd" => {
            let target = if args.len() > 1 {
                args[1].as_str()
            } else {
                match env::var("HOME") {
                    Ok(h) => {
                        return if env::set_current_dir(h).is_ok() {
                            0
                        } else {
                            1
                        }
                    }
                    Err(_) => return 0,
                }
            };
            if let Err(e) = env::set_current_dir(target) {
                eprintln!("cd: {}: {}", target, e);
                1
            } else {
                0
            }
        }
        "exit" => {
            let code = if args.len() > 1 {
                args[1].parse::<i32>().unwrap_or(0)
            } else {
                0
            };
            std::process::exit(code);
        }
        "echo" => {
            let mut out = io::stdout().lock();
            let mut newline = true;
            let mut start = 1;
            if args.len() > 1 && args[1] == "-n" {
                newline = false;
                start = 2;
            }
            for (idx, arg) in args[start..].iter().enumerate() {
                if idx > 0 {
                    let _ = out.write_all(b" ");
                }
                let _ = out.write_all(arg.as_bytes());
            }
            if newline {
                let _ = out.write_all(b"\n");
            }
            let _ = out.flush();
            0
        }
        "pwd" => {
            if let Ok(p) = env::current_dir() {
                println!("{}", p.display());
                0
            } else {
                1
            }
        }
        "exec" => {
            if args.len() > 1 {
                let _ = execute_single_command(
                    &args[1..].join(" "),
                    Stdio::inherit(),
                    Stdio::inherit(),
                );
            }
            0
        }
        ":" => 0,
        _ => 1,
    }
}

fn execute_single_command(cmd_str: &str, in_stdio: Stdio, out_stdio: Stdio) -> i32 {
    let parsed = match parse_simple_command(cmd_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sh: {}", e);
            return 1;
        }
    };

    if parsed.args.is_empty() {
        return 0;
    }

    if is_builtin(&parsed.args[0]) && parsed.stdin_file.is_none() && parsed.stdout_file.is_none() {
        return run_builtin(&parsed.args);
    }

    let stdin = if let Some(in_path) = parsed.stdin_file {
        match File::open(in_path) {
            Ok(f) => Stdio::from(f),
            Err(e) => {
                eprintln!("sh: {}", e);
                return 1;
            }
        }
    } else {
        in_stdio
    };

    let stdout = if let Some(out_path) = parsed.stdout_file {
        let res = OpenOptions::new()
            .write(true)
            .create(true)
            .append(parsed.stdout_append)
            .truncate(!parsed.stdout_append)
            .open(out_path);
        match res {
            Ok(f) => Stdio::from(f),
            Err(e) => {
                eprintln!("sh: {}", e);
                return 1;
            }
        }
    } else {
        out_stdio
    };

    if is_builtin(&parsed.args[0]) {
        return run_builtin(&parsed.args);
    }

    let mut cmd = Command::new(&parsed.args[0]);
    cmd.args(&parsed.args[1..]);
    cmd.stdin(stdin);
    cmd.stdout(stdout);

    match cmd.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("sh: {}: {}", parsed.args[0], e);
            127
        }
    }
}

fn run_shell(name: &str, args: &[OsString]) -> Result<i32> {
    let mut command_str: Option<String> = None;
    let mut script_file: Option<OsString> = None;

    let mut i = 0;
    while i < args.len() {
        let bytes = args[i].as_bytes();
        if bytes == b"-c" {
            if i + 1 < args.len() {
                command_str = Some(args[i + 1].to_string_lossy().to_string());
                break;
            }
        } else if !bytes.starts_with(b"-") {
            script_file = Some(args[i].clone());
            break;
        }
        i += 1;
    }

    if let Some(cmd) = command_str {
        return Ok(run_command_line(&cmd));
    }

    if let Some(file) = script_file {
        let f = match File::open(&file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: {}: {}", name, file.to_string_lossy(), e);
                return Ok(1);
            }
        };
        let reader = BufReader::new(f);
        let mut last_code = 0;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            last_code = run_command_line(&line);
        }
        return Ok(last_code);
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut last_code = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        last_code = run_command_line(&line);
    }

    Ok(last_code)
}

pub struct AshApplet;
impl Applet for AshApplet {
    fn name(&self) -> &'static str {
        "ash"
    }
    fn description(&self) -> &'static str {
        "Command language interpreter (ash)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_shell("ash", args)
    }
}

pub struct HushApplet;
impl Applet for HushApplet {
    fn name(&self) -> &'static str {
        "hush"
    }
    fn description(&self) -> &'static str {
        "Command language interpreter (hush)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_shell("hush", args)
    }
}

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

pub struct EdApplet;
impl Applet for EdApplet {
    fn name(&self) -> &'static str {
        "ed"
    }
    fn description(&self) -> &'static str {
        "Line-oriented text editor"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut filename: Option<PathBuf> = None;
        let mut lines: Vec<String> = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if !b.starts_with(b"-") {
                filename = Some(PathBuf::from(arg));
                break;
            }
        }

        if let Some(ref path) = filename {
            if let Ok(f) = File::open(path) {
                let reader = BufReader::new(f);
                for s in reader.lines().map_while(std::result::Result::ok) {
                    lines.push(s);
                }
                println!("{}", lines.iter().map(|l| l.len() + 1).sum::<usize>());
            }
        }

        let mut cur_line = lines.len();
        let stdin = io::stdin();
        let mut stdin_lines = stdin.lock().lines();

        while let Some(Ok(cmd_line)) = stdin_lines.next() {
            let cmd = cmd_line.trim();
            if cmd == "q" {
                break;
            } else if cmd == "p" {
                if cur_line > 0 && cur_line <= lines.len() {
                    println!("{}", lines[cur_line - 1]);
                } else if !lines.is_empty() {
                    println!("{}", lines[0]);
                } else {
                    println!("?");
                }
            } else if cmd == ",p" || cmd == "1,$p" {
                for l in &lines {
                    println!("{}", l);
                }
            } else if cmd == "a" {
                while let Some(Ok(text)) = stdin_lines.next() {
                    if text == "." {
                        break;
                    }
                    cur_line += 1;
                    if cur_line > lines.len() {
                        lines.push(text);
                    } else {
                        lines.insert(cur_line - 1, text);
                    }
                }
            } else if cmd == "i" {
                let insert_pos = if cur_line == 0 { 0 } else { cur_line - 1 };
                let mut added = 0;
                while let Some(Ok(text)) = stdin_lines.next() {
                    if text == "." {
                        break;
                    }
                    lines.insert(insert_pos + added, text);
                    added += 1;
                }
                cur_line = insert_pos + added;
            } else if cmd == "d" {
                if cur_line > 0 && cur_line <= lines.len() {
                    lines.remove(cur_line - 1);
                    if cur_line > lines.len() {
                        cur_line = lines.len();
                    }
                } else {
                    println!("?");
                }
            } else if cmd == "w" {
                if let Some(ref path) = filename {
                    if let Ok(mut f) = File::create(path) {
                        let mut total = 0;
                        for l in &lines {
                            let _ = writeln!(f, "{}", l);
                            total += l.len() + 1;
                        }
                        println!("{}", total);
                    } else {
                        println!("?");
                    }
                } else {
                    println!("?");
                }
            } else if let Ok(num) = cmd.parse::<usize>() {
                if num > 0 && num <= lines.len() {
                    cur_line = num;
                    println!("{}", lines[cur_line - 1]);
                } else {
                    println!("?");
                }
            } else {
                println!("?");
            }
        }

        Ok(0)
    }
}

pub struct ViApplet;
impl Applet for ViApplet {
    fn name(&self) -> &'static str {
        "vi"
    }
    fn description(&self) -> &'static str {
        "Screen-oriented (visual) display editor"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let file_path = args.iter().find(|a| !a.as_bytes().starts_with(b"-"));
        let mut lines = Vec::new();
        let path_buf = file_path.map(PathBuf::from);

        if let Some(ref p) = path_buf {
            if let Ok(f) = File::open(p) {
                for l in BufReader::new(f).lines().map_while(std::result::Result::ok) {
                    lines.push(l);
                }
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }

        let mut orig_termios = MaybeUninit::<libc::termios>::uninit();
        let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
        if is_tty {
            unsafe {
                libc::tcgetattr(libc::STDIN_FILENO, orig_termios.as_mut_ptr());
                let mut raw = orig_termios.assume_init();
                libc::cfmakeraw(&mut raw);
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw);
            }
        }

        let mut row = 0;
        let mut col = 0;
        let mut mode_insert = false;
        let mut status_msg = String::new();

        let render = |lines: &[String], row: usize, col: usize, msg: &str| {
            print!("\x1b[2J\x1b[H");
            for (i, l) in lines.iter().take(24).enumerate() {
                if i < lines.len() {
                    print!("{}\r\n", l);
                } else {
                    print!("~\r\n");
                }
            }
            if !msg.is_empty() {
                print!("\x1b[24;1H{}", msg);
            }
            print!("\x1b[{};{}H", row + 1, col + 1);
            let _ = io::stdout().flush();
        };

        render(&lines, row, col, &status_msg);

        let mut stdin = io::stdin();
        let mut buf = [0u8; 16];

        'editor: loop {
            let n = match stdin.read(&mut buf) {
                Ok(n) if n > 0 => n,
                _ => break,
            };

            let input = &buf[..n];

            if mode_insert {
                if input[0] == 27 {
                    mode_insert = false;
                    status_msg.clear();
                } else if input[0] == b'\r' || input[0] == b'\n' {
                    let cur_len = lines[row].len();
                    let split_at = col.min(cur_len);
                    let rest = lines[row][split_at..].to_string();
                    lines[row].truncate(split_at);
                    lines.insert(row + 1, rest);
                    row += 1;
                    col = 0;
                } else if input[0] == 127 || input[0] == 8 {
                    if col > 0 && col <= lines[row].len() {
                        lines[row].remove(col - 1);
                        col -= 1;
                    }
                } else if let Ok(s) = std::str::from_utf8(input) {
                    for ch in s.chars() {
                        if !ch.is_control() {
                            let cur_len = lines[row].len();
                            let ins_idx = col.min(cur_len);
                            lines[row].insert(ins_idx, ch);
                            col += 1;
                        }
                    }
                }
            } else {
                match input[0] {
                    b'i' => {
                        mode_insert = true;
                        status_msg = "-- INSERT --".into();
                    }
                    b'h' => col = col.saturating_sub(1),
                    b'l' => {
                        if col + 1 < lines[row].len() {
                            col += 1;
                        }
                    }
                    b'k' => {
                        row = row.saturating_sub(1);
                        col = col.min(lines[row].len());
                    }
                    b'j' => {
                        if row + 1 < lines.len() {
                            row += 1;
                            col = col.min(lines[row].len());
                        }
                    }
                    b'd' if n > 1 && buf[1] == b'd' => {
                        if lines.len() > 1 {
                            lines.remove(row);
                            if row >= lines.len() {
                                row = lines.len() - 1;
                            }
                        } else {
                            lines[0].clear();
                        }
                        col = 0;
                    }
                    b':' => {
                        if is_tty {
                            unsafe {
                                libc::tcsetattr(
                                    libc::STDIN_FILENO,
                                    libc::TCSANOW,
                                    orig_termios.as_ptr(),
                                );
                            }
                        }
                        print!("\x1b[24;1H:");
                        let _ = io::stdout().flush();
                        let mut cmd = String::new();
                        let _ = io::stdin().read_line(&mut cmd);
                        let cmd = cmd.trim();

                        if is_tty {
                            unsafe {
                                let mut raw = orig_termios.assume_init();
                                libc::cfmakeraw(&mut raw);
                                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw);
                            }
                        }

                        if cmd == "q" || cmd == "q!" {
                            break 'editor;
                        } else if cmd == "w" || cmd == "wq" {
                            if let Some(ref p) = path_buf {
                                if let Ok(mut f) = File::create(p) {
                                    for l in &lines {
                                        let _ = writeln!(f, "{}", l);
                                    }
                                }
                            }
                            if cmd == "wq" {
                                break 'editor;
                            }
                        }
                    }
                    _ => {}
                }
            }

            render(&lines, row, col, &status_msg);
        }

        if is_tty {
            unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, orig_termios.as_ptr());
            }
        }
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();

        Ok(0)
    }
}

fn run_pager(is_less: bool, args: &[OsString]) -> Result<i32> {
    let mut files = Vec::new();
    for arg in args {
        let b = arg.as_bytes();
        if !b.starts_with(b"-") {
            files.push(PathBuf::from(arg));
        }
    }

    let mut lines = Vec::new();
    if files.is_empty() {
        let stdin = io::stdin();
        for l in stdin.lock().lines().map_while(std::result::Result::ok) {
            lines.push(l);
        }
    } else {
        for f in &files {
            if let Ok(file) = File::open(f) {
                for l in BufReader::new(file)
                    .lines()
                    .map_while(std::result::Result::ok)
                {
                    lines.push(l);
                }
            }
        }
    }

    let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
    if !is_tty {
        for l in &lines {
            println!("{}", l);
        }
        return Ok(0);
    }

    let mut orig_termios = MaybeUninit::<libc::termios>::uninit();
    unsafe {
        libc::tcgetattr(libc::STDIN_FILENO, orig_termios.as_mut_ptr());
        let mut raw = orig_termios.assume_init();
        libc::cfmakeraw(&mut raw);
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw);
    }

    let term_height = 24usize;
    let mut top = 0usize;

    let display = |top: usize| {
        print!("\x1b[2J\x1b[H");
        let end = (top + term_height - 1).min(lines.len());
        for line in lines.iter().take(end).skip(top) {
            print!("{}\r\n", line);
        }
        print!(":");
        let _ = io::stdout().flush();
    };

    display(top);

    let mut stdin = io::stdin();
    let mut buf = [0u8; 4];

    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 {
            break;
        }
        match buf[0] {
            b'q' | b'Q' => break,
            b' ' => {
                if top + term_height - 1 < lines.len() {
                    top += term_height - 1;
                }
                display(top);
            }
            b'\r' | b'\n' => {
                if top + 1 < lines.len() {
                    top += 1;
                }
                display(top);
            }
            b'b' if is_less => {
                top = top.saturating_sub(term_height - 1);
                display(top);
            }
            _ => {}
        }
    }

    unsafe {
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, orig_termios.as_ptr());
    }
    print!("\r\x1b[K");
    let _ = io::stdout().flush();

    Ok(0)
}

pub struct LessApplet;
impl Applet for LessApplet {
    fn name(&self) -> &'static str {
        "less"
    }
    fn description(&self) -> &'static str {
        "Opposite of more: view file contents with backward scrolling"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_pager(true, args)
    }
}

pub struct MoreApplet;
impl Applet for MoreApplet {
    fn name(&self) -> &'static str {
        "more"
    }
    fn description(&self) -> &'static str {
        "View file contents one screenful at a time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_pager(false, args)
    }
}

pub struct ManApplet;
impl Applet for ManApplet {
    fn name(&self) -> &'static str {
        "man"
    }
    fn description(&self) -> &'static str {
        "Format and display the on-line manual pages"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let topic = match args.iter().find(|a| !a.as_bytes().starts_with(b"-")) {
            Some(t) => t.to_string_lossy(),
            None => {
                eprintln!("What manual page do you want?");
                return Ok(1);
            }
        };

        let man_dirs = [
            "/usr/share/man",
            "/usr/local/share/man",
            "/usr/man",
            "/nix/var/nix/profiles/default/share/man",
        ];
        let sections = ["man1", "man8", "man5", "man7", "man2", "man3"];

        let mut target_file: Option<PathBuf> = None;
        for dir in &man_dirs {
            for sec in &sections {
                let p = Path::new(dir).join(sec).join(format!("{}.1", topic));
                if p.exists() {
                    target_file = Some(p);
                    break;
                }
                let p_gz = Path::new(dir).join(sec).join(format!("{}.1.gz", topic));
                if p_gz.exists() {
                    target_file = Some(p_gz);
                    break;
                }
            }
            if target_file.is_some() {
                break;
            }
        }

        let file_path = match target_file {
            Some(p) => p,
            None => {
                eprintln!("No manual entry for {}", topic);
                return Ok(1);
            }
        };

        let content = if file_path.extension().and_then(|e| e.to_str()) == Some("gz") {
            let out = Command::new("gzip")
                .arg("-dc")
                .arg(&file_path)
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            String::from_utf8_lossy(&out).to_string()
        } else {
            fs::read_to_string(&file_path).unwrap_or_default()
        };

        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(content.as_bytes());
        let _ = stdout.flush();

        Ok(0)
    }
}

pub struct DcApplet;
impl Applet for DcApplet {
    fn name(&self) -> &'static str {
        "dc"
    }
    fn description(&self) -> &'static str {
        "Reverse-polish arbitrary precision desk calculator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" && i + 1 < args.len() {
                exprs.push(args[i + 1].to_string_lossy().to_string());
                i += 2;
                continue;
            }
            i += 1;
        }

        let mut stack: Vec<i64> = Vec::new();

        let mut execute_tok = |tok: &str| match tok {
            "+" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a + b);
                }
            }
            "-" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
            }
            "*" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a * b);
                }
            }
            "/" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b != 0 {
                        stack.push(a / b);
                    } else {
                        eprintln!("dc: divide by zero");
                    }
                }
            }
            "%" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b != 0 {
                        stack.push(a % b);
                    }
                }
            }
            "p" => {
                if let Some(top) = stack.last() {
                    println!("{}", top);
                } else {
                    eprintln!("dc: stack empty");
                }
            }
            "f" => {
                for v in stack.iter().rev() {
                    println!("{}", v);
                }
            }
            num_str => {
                if let Ok(n) = num_str.parse::<i64>() {
                    stack.push(n);
                }
            }
        };

        if !exprs.is_empty() {
            for expr in exprs {
                for tok in expr.split_whitespace() {
                    execute_tok(tok);
                }
            }
        } else {
            let stdin = io::stdin();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                for tok in line.split_whitespace() {
                    execute_tok(tok);
                }
            }
        }

        Ok(0)
    }
}

pub struct BcApplet;
impl Applet for BcApplet {
    fn name(&self) -> &'static str {
        "bc"
    }
    fn description(&self) -> &'static str {
        "Arbitrary precision calculator language"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" && i + 1 < args.len() {
                exprs.push(args[i + 1].to_string_lossy().to_string());
                i += 2;
                continue;
            }
            i += 1;
        }

        let eval_infix = |expr: &str| {
            if let Some(res) = eval_simple_math(expr) {
                println!("{}", res);
            }
        };

        if !exprs.is_empty() {
            for e in exprs {
                eval_infix(&e);
            }
        } else {
            let stdin = io::stdin();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                let line = line.trim();
                if line == "quit" {
                    break;
                }
                if !line.is_empty() {
                    eval_infix(line);
                }
            }
        }

        Ok(0)
    }
}

fn eval_simple_math(expr: &str) -> Option<i64> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_digit() {
            let mut num = 0i64;
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num = num * 10 + (d as i64 - '0' as i64);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Num(num));
        } else {
            match c {
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Star),
                '/' => tokens.push(Token::Slash),
                '%' => tokens.push(Token::Percent),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                _ => {}
            }
            chars.next();
        }
    }

    #[derive(Debug, PartialEq)]
    enum Token {
        Num(i64),
        Plus,
        Minus,
        Star,
        Slash,
        Percent,
        LParen,
        RParen,
    }

    let mut output = Vec::new();
    let mut ops = Vec::new();

    let precedence = |t: &Token| match t {
        Token::Star | Token::Slash | Token::Percent => 2,
        Token::Plus | Token::Minus => 1,
        _ => 0,
    };

    for tok in tokens {
        match tok {
            Token::Num(n) => output.push(Token::Num(n)),
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent => {
                while let Some(top) = ops.last() {
                    if *top != Token::LParen && precedence(top) >= precedence(&tok) {
                        output.push(ops.pop().unwrap());
                    } else {
                        break;
                    }
                }
                ops.push(tok);
            }
            Token::LParen => ops.push(tok),
            Token::RParen => {
                while let Some(top) = ops.pop() {
                    if top == Token::LParen {
                        break;
                    }
                    output.push(top);
                }
            }
        }
    }
    while let Some(top) = ops.pop() {
        output.push(top);
    }

    let mut stack = Vec::new();
    for tok in output {
        match tok {
            Token::Num(n) => stack.push(n),
            Token::Plus => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a + b);
            }
            Token::Minus => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a - b);
            }
            Token::Star => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a * b);
            }
            Token::Slash => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                if b == 0 {
                    return None;
                }
                stack.push(a / b);
            }
            Token::Percent => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                if b == 0 {
                    return None;
                }
                stack.push(a % b);
            }
            _ => {}
        }
    }

    stack.pop()
}

pub struct DoubleLBracketApplet;
impl Applet for DoubleLBracketApplet {
    fn name(&self) -> &'static str {
        "[["
    }
    fn description(&self) -> &'static str {
        "Extended shell conditional expression evaluator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut items = Vec::new();
        for arg in args {
            let s = arg.to_string_lossy();
            if s != "]]" {
                items.push(s.to_string());
            }
        }

        if items.is_empty() {
            return Ok(1);
        }

        if let Some(idx) = items.iter().position(|x| x == "||") {
            let left_ok = eval_extended_test(&items[..idx]);
            if left_ok {
                return Ok(0);
            }
            let right_ok = eval_extended_test(&items[idx + 1..]);
            return Ok(if right_ok { 0 } else { 1 });
        }

        if let Some(idx) = items.iter().position(|x| x == "&&") {
            let left_ok = eval_extended_test(&items[..idx]);
            if !left_ok {
                return Ok(1);
            }
            let right_ok = eval_extended_test(&items[idx + 1..]);
            return Ok(if right_ok { 0 } else { 1 });
        }

        let res = eval_extended_test(&items);
        Ok(if res { 0 } else { 1 })
    }
}

fn eval_extended_test(items: &[String]) -> bool {
    if items.is_empty() {
        return false;
    }
    if items.len() == 1 {
        return !items[0].is_empty();
    }
    if items.len() == 2 {
        let op = &items[0];
        let arg = &items[1];
        return match op.as_str() {
            "-z" => arg.is_empty(),
            "-n" => !arg.is_empty(),
            "-f" => Path::new(arg).is_file(),
            "-d" => Path::new(arg).is_dir(),
            "-e" => Path::new(arg).exists(),
            "!" => !eval_extended_test(&items[1..]),
            _ => false,
        };
    }
    if items.len() == 3 {
        let lhs = &items[0];
        let op = &items[1];
        let rhs = &items[2];
        return match op.as_str() {
            "==" | "=" => lhs == rhs,
            "!=" => lhs != rhs,
            "-eq" => lhs.parse::<i64>().unwrap_or(0) == rhs.parse::<i64>().unwrap_or(0),
            "-ne" => lhs.parse::<i64>().unwrap_or(0) != rhs.parse::<i64>().unwrap_or(0),
            "-lt" => lhs.parse::<i64>().unwrap_or(0) < rhs.parse::<i64>().unwrap_or(0),
            "-le" => lhs.parse::<i64>().unwrap_or(0) <= rhs.parse::<i64>().unwrap_or(0),
            "-gt" => lhs.parse::<i64>().unwrap_or(0) > rhs.parse::<i64>().unwrap_or(0),
            "-ge" => lhs.parse::<i64>().unwrap_or(0) >= rhs.parse::<i64>().unwrap_or(0),
            _ => false,
        };
    }
    false
}
