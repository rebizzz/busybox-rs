use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub fn run_pager(is_less: bool, args: &[OsString]) -> Result<i32> {
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
