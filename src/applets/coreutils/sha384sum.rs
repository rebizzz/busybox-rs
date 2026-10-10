use super::common::*;
use crate::core::Result;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub trait Upd {
    fn upd(&mut self, d: &[u8]);
    fn fin(&mut self) -> Vec<u8>;
}
pub fn hash_bytes(h: &mut dyn Upd, p: &OsStr) -> std::io::Result<Vec<u8>> {
    if p.as_bytes() == b"-" {
        let mut buf = [0u8; 8192];
        let mut inp = std::io::stdin().lock();
        loop {
            let n = inp.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.upd(&buf[..n]);
        }
    } else {
        let mut f = File::open(Path::new(p))?;
        let mut buf = [0u8; 8192];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.upd(&buf[..n]);
        }
    }
    Ok(h.fin())
}

pub fn sum_main(name: &str, args: &[OsString], mk: impl Fn() -> Box<dyn Upd>) -> Result<i32> {
    let (mut check, mut silent, mut warn, mut bin) = (false, false, false, false);
    let mut files: Vec<OsString> = Vec::new();
    let mut nopt = true;
    for a in args {
        let b = ab(a);
        if nopt && b == b"--" {
            nopt = false;
        } else if nopt && b.len() > 1 && b[0] == b'-' && b != b"-" {
            for &c in &b[1..] {
                match c {
                    b'c' => check = true,
                    b's' => silent = true,
                    b'w' => warn = true,
                    b'b' => bin = true,
                    b't' => bin = false,
                    _ => {
                        eprintln!("{}: invalid option '{}'", name, lossy(a));
                        return Ok(1);
                    }
                }
            }
        } else {
            files.push(a.clone());
        }
    }
    if (silent || warn) && !check {
        eprintln!("{}: -s and -w require -c", name);
        return Ok(1);
    }
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let mut rc = 0;
    let mut out = wlock();
    if check {
        let (mut tot, mut bad) = (0, 0);
        for f in &files {
            let data = match read_all(f.as_os_str()) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("{}: can't open '{}': {}", name, lossy(f), e);
                    rc = 1;
                    continue;
                }
            };
            for line in data.split(|&c| c == b'\n') {
                let line = if line.last() == Some(&b'\r') {
                    &line[..line.len() - 1]
                } else {
                    line
                };
                if line.is_empty() {
                    continue;
                }
                let sp = match line.iter().position(|&c| c == b' ' || c == b'*') {
                    Some(p) => p,
                    None => {
                        if warn {
                            eprintln!("{}: invalid format", name);
                        }
                        tot += 1;
                        bad += 1;
                        rc = 1;
                        continue;
                    }
                };
                let (exp, fpart) = (
                    &line[..sp],
                    line[sp + 1..].strip_prefix(b" ").unwrap_or(&line[sp + 1..]),
                );

                let fpart = fpart.strip_prefix(b"*").unwrap_or(fpart);

                let fname = Path::new(OsStr::from_bytes(fpart));
                tot += 1;
                match hash_bytes(mk().as_mut(), fname.as_os_str()) {
                    Ok(d) => {
                        let got = hex_str(&d);
                        if got.eq_ignore_ascii_case(&String::from_utf8_lossy(exp)) {
                            if !silent {
                                let _ = writeln!(out, "{}: OK", String::from_utf8_lossy(fpart));
                            }
                        } else {
                            if !silent {
                                let _ = writeln!(out, "{}: FAILED", String::from_utf8_lossy(fpart));
                            }
                            bad += 1;
                            rc = 1;
                        }
                    }
                    Err(_) => {
                        if !silent {
                            let _ = writeln!(out, "{}: FAILED", String::from_utf8_lossy(fpart));
                        }
                        bad += 1;
                        rc = 1;
                    }
                }
            }
        }
        if bad > 0 && !silent {
            eprintln!(
                "{}: WARNING: {} of {} checksums did NOT match",
                name, bad, tot
            );
        }
        if tot == 0 {
            eprintln!("{}: no checksum lines found", name);
            rc = 1;
        }
        return Ok(rc);
    }
    let mark = if bin { " *" } else { "  " };
    for f in &files {
        match hash_bytes(mk().as_mut(), f.as_os_str()) {
            Ok(d) => {
                let _ = writeln!(out, "{}{}{}", hex_str(&d), mark, lossy(f));
            }
            Err(e) => {
                eprintln!("{}: can't open '{}': {}", name, lossy(f), e);
                rc = 1;
            }
        }
    }
    Ok(rc)
}

const K512: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];
struct Sha384 {
    st: [u64; 8],
    n: u128,
    buf: [u8; 128],
}
impl Sha384 {
    fn new() -> Self {
        Self {
            st: [
                0xcbbb9d5dc1059ed8,
                0x629a292a367cd507,
                0x9159015a3070dd17,
                0x152fecd8f70e5939,
                0x67332667ffc00b31,
                0x8eb44a8768581511,
                0xdb0c2e0d64f98fa7,
                0x47b5481dbefa4fa4,
            ],
            n: 0,
            buf: [0u8; 128],
        }
    }
    fn blk(st: &mut [u64; 8], b: &[u8; 128]) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes(b[i * 8..i * 8 + 8].try_into().unwrap_or([0u8; 8]));
        }
        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b2, mut c, mut d, mut e, mut f, mut g, mut h) =
            (st[0], st[1], st[2], st[3], st[4], st[5], st[6], st[7]);
        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K512[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let t2 = s0.wrapping_add((a & b2) ^ (a & c) ^ (b2 & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b2;
            b2 = a;
            a = t1.wrapping_add(t2);
        }
        st[0] = st[0].wrapping_add(a);
        st[1] = st[1].wrapping_add(b2);
        st[2] = st[2].wrapping_add(c);
        st[3] = st[3].wrapping_add(d);
        st[4] = st[4].wrapping_add(e);
        st[5] = st[5].wrapping_add(f);
        st[6] = st[6].wrapping_add(g);
        st[7] = st[7].wrapping_add(h);
    }
    fn pad(&mut self) -> [u8; 48] {
        let bit = self.n.wrapping_mul(8);
        let idx = (self.n & 0x7f) as usize;
        let pl = if idx < 112 { 112 - idx } else { 240 - idx };
        let mut p = [0u8; 256];
        p[0] = 0x80;
        let mut k = 0;
        while k < pl {
            let n = (pl - k).min(128 - ((self.n as usize) & 0x7f));
            let off = (self.n & 0x7f) as usize;
            if off == 0 && n == 128 {
                let blk: &[u8; 128] = p[k..k + 128].try_into().unwrap();
                Self::blk(&mut self.st, blk);
            } else {
                self.buf[off..off + n].copy_from_slice(&p[k..k + n]);
                if off + n == 128 {
                    let c = self.buf;
                    Self::blk(&mut self.st, &c);
                }
            }
            self.n += n as u128;
            k += n;
        }
        let lb = bit.to_be_bytes();
        let off = (self.n & 0x7f) as usize;
        self.buf[off..off + 16].copy_from_slice(&lb);
        let c = self.buf;
        Self::blk(&mut self.st, &c);
        let mut o = [0u8; 48];
        for (i, v) in self.st.iter().take(6).enumerate() {
            o[i * 8..i * 8 + 8].copy_from_slice(&v.to_be_bytes());
        }
        o
    }
}
impl Upd for Sha384 {
    fn upd(&mut self, d: &[u8]) {
        let mut i = 0;
        let idx = (self.n & 0x7f) as usize;
        if idx > 0 {
            let sp = 128 - idx;
            if d.len() >= sp {
                self.buf[idx..128].copy_from_slice(&d[..sp]);
                let c = self.buf;
                Self::blk(&mut self.st, &c);
                self.n += sp as u128;
                i = sp;
            } else {
                self.buf[idx..idx + d.len()].copy_from_slice(d);
                self.n += d.len() as u128;
                return;
            }
        }
        while i + 128 <= d.len() {
            let blk: &[u8; 128] = d[i..i + 128].try_into().unwrap();
            Self::blk(&mut self.st, blk);
            self.n += 128;
            i += 128;
        }
        if i < d.len() {
            self.buf[..d.len() - i].copy_from_slice(&d[i..]);
            self.n += (d.len() - i) as u128;
        }
    }
    fn fin(&mut self) -> Vec<u8> {
        self.pad().to_vec()
    }
}
applet!(
    Sha384sumApplet,
    "sha384sum",
    "Print or check SHA384 checksums",
    run_sha384sum
);
fn run_sha384sum(args: &[OsString]) -> Result<i32> {
    sum_main("sha384sum", args, || Box::new(Sha384::new()))
}

