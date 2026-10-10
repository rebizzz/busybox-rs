use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};

applet!(UuidgenApplet, "uuidgen", "Generate UUIDs", run_uuidgen);
fn run_uuidgen(args: &[OsString]) -> Result<i32> {
    let (mut n, mut v1) = (1u32, false);
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-r" || b == b"--random" {
            v1 = false;
        } else if b == b"-t" || b == b"--time" {
            v1 = true;
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("uuidgen: -n needs an argument");
                return Ok(1);
            }
            n = lossy(&args[i]).parse().unwrap_or(1).min(1000);
        } else if b.starts_with(b"-n") && b.len() > 2 {
            n = String::from_utf8_lossy(&b[2..])
                .parse()
                .unwrap_or(1)
                .min(1000);
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("uuidgen: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        }
        i += 1;
    }
    let mut rnd = File::open("/dev/urandom").ok();
    let mut tbuf = [0u8; 16];
    let mut out = wlock();
    let mut line = Vec::with_capacity(40);
    for _ in 0..n.max(1) {
        if let Some(f) = rnd.as_mut() {
            if f.read_exact(&mut tbuf).is_err() {
                let mut x = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() as u64)
                    .unwrap_or(0x12345);
                for b in &mut tbuf {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    *b = x as u8;
                }
            }
        }
        if v1 {
            let ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            tbuf[0..8].copy_from_slice(&ns.to_be_bytes());
            tbuf[6] = (tbuf[6] & 0x0f) | 0x10;
        } else {
            tbuf[6] = (tbuf[6] & 0x0f) | 0x40;
        }
        tbuf[8] = (tbuf[8] & 0x3f) | 0x80;
        line.clear();
        for (k, &bb) in tbuf.iter().enumerate() {
            let _ = write!(line, "{:02x}", bb);
            if matches!(k, 3 | 5 | 7 | 9) {
                line.push(b'-');
            }
        }
        line.push(b'\n');
        let _ = out.write_all(&line);
    }
    Ok(0)
}

fn ipc_key_rm(kind: u8, id_or_key: &str, by_key: bool) -> std::io::Result<()> {
    unsafe {
        if by_key {
            let key: i32 = if let Some(h) = id_or_key
                .strip_prefix("0x")
                .or_else(|| id_or_key.strip_prefix("0X"))
            {
                i32::from_str_radix(h, 16)
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad key"))?
            } else {
                id_or_key
                    .parse()
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad key"))?
            };
            let r = match kind {
                b'm' => {
                    let id = libc::shmget(key, 0, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut())
                }
                b's' => {
                    let id = libc::semget(key, 0, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::semctl(id, 0, libc::IPC_RMID)
                }
                _ => {
                    let id = libc::msgget(key, 0);
                    if id < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut())
                }
            };
            if r < 0 {
                return Err(std::io::Error::last_os_error());
            }
        } else {
            let id: i32 = id_or_key
                .parse()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad id"))?;
            let r = match kind {
                b'm' => libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()),
                b's' => libc::semctl(id, 0, libc::IPC_RMID),
                _ => libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut()),
            };
            if r < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
