use crate::core::Result;
use std::env;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
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

pub fn run_shell(name: &str, args: &[OsString]) -> Result<i32> {
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
