use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

fn rx_crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

pub struct RxApplet;
impl Applet for RxApplet {
    fn name(&self) -> &'static str {
        "rx"
    }
    fn description(&self) -> &'static str {
        "Receive file via XMODEM (128/1K blocks, CRC or checksum)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dest: Option<PathBuf> = None;
        for a in args {
            if !ab(a).starts_with(b"-") {
                dest = Some(PathBuf::from(a));
            }
        }
        let dest = match dest {
            Some(d) => d,
            None => {
                eprintln!("rx: missing destination file");
                return Ok(1);
            }
        };
        let mut out = match File::create(&dest) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("rx: {}: {e}", dest.display());
                return Ok(1);
            }
        };
        let stdin = std::io::stdin();
        let mut inp = stdin.lock();
        let stdout = std::io::stdout();
        let mut ctl = stdout.lock();

        let mut use_crc = true;
        let _ = ctl.write_all(b"C");
        let _ = ctl.flush();
        let mut buf = [0u8; 1030];
        let mut expected: u8 = 1;
        let mut retries = 0u32;
        let mut pending_block: Vec<u8> = Vec::new();
        loop {
            if retries > 25 {
                eprintln!("rx: too many errors, aborting");
                let _ = ctl.write_all(&[0x18]);
                return Ok(1);
            }
            let mut first = [0u8; 1];
            let n = match inp.read(&mut first) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("rx: {e}");
                    return Ok(1);
                }
            };
            if n == 0 {
                retries += 1;
                let _ = ctl.write_all(&[0x15]);
                continue;
            }
            match first[0] {
                0x04 => {
                    // EOT: remove padding ^Z from previous block if at least 3 trailing ^Z
                    if pending_block.len() >= 3
                        && pending_block[pending_block.len() - 1] == 0x1A
                        && pending_block[pending_block.len() - 2] == 0x1A
                        && pending_block[pending_block.len() - 3] == 0x1A
                    {
                        while pending_block.last() == Some(&0x1A) {
                            pending_block.pop();
                        }
                    }
                    if out.write_all(&pending_block).is_err() {
                        eprintln!("rx: write error");
                        return Ok(1);
                    }
                    let _ = ctl.write_all(&[0x06]);
                    break;
                }
                0x18 => {
                    eprintln!("rx: cancelled by sender");
                    return Ok(1);
                }
                0x01 | 0x02 => {
                    // Write previously received block before processing new one
                    if !pending_block.is_empty() {
                        if out.write_all(&pending_block).is_err() {
                            eprintln!("rx: write error");
                            return Ok(1);
                        }
                        pending_block.clear();
                    }
                    let blklen = if first[0] == 0x01 { 128 } else { 1024 };
                    let need = blklen + 2 + if use_crc { 2 } else { 1 };
                    let mut got = 0;
                    while got < need {
                        match inp.read(&mut buf[got..need]) {
                            Ok(0) => break,
                            Ok(n) => got += n,
                            Err(e) => {
                                eprintln!("rx: {e}");
                                return Ok(1);
                            }
                        }
                    }
                    if got < need {
                        retries += 1;
                        let _ = ctl.write_all(&[0x15]);
                        continue;
                    }
                    let seq = buf[0];
                    let seqc = buf[1];
                    if seq != expected || seqc != expected ^ 0xFF {
                        if seq == expected.wrapping_sub(1) {
                            let _ = ctl.write_all(&[0x06]);
                        } else {
                            retries += 1;
                            let _ = ctl.write_all(&[0x15]);
                        }
                        continue;
                    }
                    let payload = &buf[2..2 + blklen];
                    let ok = if use_crc {
                        let want = ((buf[2 + blklen] as u16) << 8) | buf[2 + blklen + 1] as u16;
                        rx_crc16(payload) == want
                    } else {
                        let want = buf[2 + blklen];
                        payload.iter().fold(0u8, |a, &b| a.wrapping_add(b)) == want
                    };
                    if !ok {
                        use_crc = false;
                        retries += 1;
                        let _ = ctl.write_all(&[0x15]);
                        continue;
                    }
                    pending_block = payload.to_vec();
                    expected = expected.wrapping_add(1);
                    retries = 0;
                    let _ = ctl.write_all(&[0x06]);
                }
                _ => {
                    if retries == 2 {
                        use_crc = false;
                    }
                    let _ = ctl.write_all(&[0x15]);
                    retries += 1;
                }
            }
            let _ = ctl.flush();
        }
        Ok(0)
    }
}
