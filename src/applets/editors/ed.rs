use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

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
