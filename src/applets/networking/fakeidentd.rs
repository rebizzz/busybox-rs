use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::os::unix::ffi::OsStrExt;

pub struct FakeidentdApplet;

impl Applet for FakeidentdApplet {
    fn name(&self) -> &'static str {
        "fakeidentd"
    }
    fn description(&self) -> &'static str {
        "Fake identd daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ident_user = "nobody".to_string();
        let mut port = 113u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(113);
            } else if !b.starts_with(b"-") {
                ident_user = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("fakeidentd: bind: {}", e);
                return Ok(1);
            }
        };

        for mut stream in listener.incoming().flatten() {
            let user = ident_user.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                });
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let parts: Vec<&str> = line.trim().split(',').collect();
                    if parts.len() == 2 {
                        let resp = format!(
                            "{}, {} : USERID : UNIX : {}\r\n",
                            parts[0].trim(),
                            parts[1].trim(),
                            user
                        );
                        let _ = stream.write_all(resp.as_bytes());
                    }
                }
            });
        }
        Ok(0)
    }
}
