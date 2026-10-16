use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

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
