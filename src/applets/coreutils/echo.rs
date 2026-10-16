use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct EchoApplet;
impl Applet for EchoApplet {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn description(&self) -> &'static str {
        "Display a line of text"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_newline = false;
        let mut interpret_escapes = false;
        let mut idx = 0;

        while idx < args.len() {
            let arg = &args[idx];
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 {
                let rest = &bytes[1..];
                if rest.iter().all(|&b| b == b'n' || b == b'e' || b == b'E') {
                    for &b in rest {
                        match b {
                            b'n' => no_newline = true,
                            b'e' => interpret_escapes = true,
                            b'E' => interpret_escapes = false,
                            _ => {}
                        }
                    }
                    idx += 1;
                    continue;
                }
            }
            break;
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut first = true;

        for arg in &args[idx..] {
            if !first {
                handle.write_all(b" ")?;
            }
            first = false;

            let bytes = arg.as_bytes();
            if interpret_escapes {
                let mut i = 0;
                let mut stop = false;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                        match bytes[i] {
                            b'a' => {
                                handle.write_all(b"\x07")?;
                            }
                            b'b' => {
                                handle.write_all(b"\x08")?;
                            }
                            b'c' => {
                                stop = true;
                                break;
                            }
                            b'e' | b'E' => {
                                handle.write_all(b"\x1b")?;
                            }
                            b'f' => {
                                handle.write_all(b"\x0c")?;
                            }
                            b'n' => {
                                handle.write_all(b"\n")?;
                            }
                            b'r' => {
                                handle.write_all(b"\r")?;
                            }
                            b't' => {
                                handle.write_all(b"\t")?;
                            }
                            b'v' => {
                                handle.write_all(b"\x0b")?;
                            }
                            b'\\' => {
                                handle.write_all(b"\\")?;
                            }
                            b'0'..=b'7' => {
                                let mut val: u32 = (bytes[i] - b'0') as u32;
                                let mut count = 1;
                                if val == 0 {
                                    while count < 4
                                        && i + 1 < bytes.len()
                                        && bytes[i + 1] >= b'0'
                                        && bytes[i + 1] <= b'7'
                                    {
                                        i += 1;
                                        val = (val << 3) + (bytes[i] - b'0') as u32;
                                        count += 1;
                                    }
                                } else {
                                    while count < 3
                                        && i + 1 < bytes.len()
                                        && bytes[i + 1] >= b'0'
                                        && bytes[i + 1] <= b'7'
                                    {
                                        i += 1;
                                        val = (val << 3) + (bytes[i] - b'0') as u32;
                                        count += 1;
                                    }
                                }
                                handle.write_all(&[(val & 0xFF) as u8])?;
                            }
                            b'x' => {
                                let mut val = 0u32;
                                let mut found = false;
                                while i + 1 < bytes.len()
                                    && (bytes[i + 1] as char).is_ascii_hexdigit()
                                {
                                    i += 1;
                                    val = (val << 4)
                                        + match bytes[i] {
                                            b'0'..=b'9' => (bytes[i] - b'0') as u32,
                                            b'a'..=b'f' => (bytes[i] - b'a' + 10) as u32,
                                            b'A'..=b'F' => (bytes[i] - b'A' + 10) as u32,
                                            _ => 0,
                                        };
                                    found = true;
                                }
                                if found {
                                    handle.write_all(&[(val & 0xFF) as u8])?;
                                } else {
                                    handle.write_all(b"\\x")?;
                                }
                            }
                            other => {
                                handle.write_all(&[b'\\', other])?;
                            }
                        }
                    } else {
                        handle.write_all(&[bytes[i]])?;
                    }
                    i += 1;
                }
                if stop {
                    return Ok(0);
                }
            } else {
                handle.write_all(bytes)?;
            }
        }

        if !no_newline {
            handle.write_all(b"\n")?;
        }
        handle.flush()?;
        Ok(0)
    }
}
