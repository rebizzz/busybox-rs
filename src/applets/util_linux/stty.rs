use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::mem::MaybeUninit;

pub struct SttyApplet;
impl Applet for SttyApplet {
    fn name(&self) -> &'static str {
        "stty"
    }
    fn description(&self) -> &'static str {
        "Change and print terminal line settings"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut termios = MaybeUninit::<libc::termios>::uninit();
        let fd = libc::STDIN_FILENO;

        if unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) } != 0 {
            eprintln!("stty: standard input: Inappropriate ioctl for device");
            return Ok(1);
        }
        let mut termios = unsafe { termios.assume_init() };

        if args.is_empty() {
            println!("speed 38400 baud; line = 0;");
            return Ok(0);
        }

        let mut changed = false;
        for arg in args {
            let s = arg.to_string_lossy();
            match s.as_ref() {
                "-a" | "--all" => {
                    let mut ws = MaybeUninit::<Winsize>::uninit();
                    let (rows, cols) =
                        if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                            let ws = unsafe { ws.assume_init() };
                            (ws.ws_row, ws.ws_col)
                        } else {
                            (24, 80)
                        };
                    println!(
                        "speed 38400 baud; rows {}; columns {}; line = 0;",
                        rows, cols
                    );
                    return Ok(0);
                }
                "size" => {
                    let mut ws = MaybeUninit::<Winsize>::uninit();
                    if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                        let ws = unsafe { ws.assume_init() };
                        println!("{} {}", ws.ws_row, ws.ws_col);
                    } else {
                        println!("24 80");
                    }
                    return Ok(0);
                }
                "raw" => {
                    unsafe { libc::cfmakeraw(&mut termios) };
                    changed = true;
                }
                "echo" => {
                    termios.c_lflag |= libc::ECHO;
                    changed = true;
                }
                "-echo" => {
                    termios.c_lflag &= !libc::ECHO;
                    changed = true;
                }
                "icanon" => {
                    termios.c_lflag |= libc::ICANON;
                    changed = true;
                }
                "-icanon" => {
                    termios.c_lflag &= !libc::ICANON;
                    changed = true;
                }
                "isig" => {
                    termios.c_lflag |= libc::ISIG;
                    changed = true;
                }
                "-isig" => {
                    termios.c_lflag &= !libc::ISIG;
                    changed = true;
                }
                _ => {}
            }
        }

        if changed {
            unsafe {
                libc::tcsetattr(fd, libc::TCSANOW, &termios);
            }
        }

        Ok(0)
    }
}
