use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

macro_rules! applet {
    ($t:ident, $n:literal, $d:literal, $f:ident) => {
        pub struct $t;
        impl Applet for $t {
            fn name(&self) -> &'static str {
                $n
            }
            fn description(&self) -> &'static str {
                $d
            }
            fn run(&self, args: &[OsString]) -> Result<i32> {
                $f(args)
            }
        }
    };
}

fn ab(a: &OsString) -> &[u8] {
    a.as_os_str().as_bytes()
}
fn lossy(a: &OsString) -> String {
    a.as_os_str().to_string_lossy().into_owned()
}
fn read_all(p: &OsStr) -> std::io::Result<Vec<u8>> {
    if p.as_bytes() == b"-" {
        let mut v = Vec::new();
        std::io::stdin().lock().read_to_end(&mut v)?;
        Ok(v)
    } else {
        let mut v = Vec::new();
        File::open(Path::new(p))?.read_to_end(&mut v)?;
        Ok(v)
    }
}
fn hex_up(out: &mut Vec<u8>, b: &[u8]) {
    const H: &[u8; 16] = b"0123456789abcdef";
    out.reserve(b.len() * 2);
    for &x in b {
        out.push(H[(x >> 4) as usize]);
        out.push(H[(x & 15) as usize]);
    }
}
fn hex_str(b: &[u8]) -> String {
    let mut v = Vec::with_capacity(b.len() * 2);
    hex_up(&mut v, b);
    String::from_utf8(v).unwrap_or_default()
}
fn wlock() -> std::io::StdoutLock<'static> {
    std::io::stdout().lock()
}

trait Upd {
    fn upd(&mut self, d: &[u8]);
    fn fin(&mut self) -> Vec<u8>;
}
fn hash_bytes(h: &mut dyn Upd, p: &OsStr) -> std::io::Result<Vec<u8>> {
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

fn sum_main(name: &str, args: &[OsString], mk: impl Fn() -> Box<dyn Upd>) -> Result<i32> {
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

fn keccakf(s: &mut [u64; 25]) {
    const RC: [u64; 24] = [
        0x0000000000000001,
        0x0000000000008082,
        0x800000000000808a,
        0x8000000080008000,
        0x000000000000808b,
        0x0000000080000001,
        0x8000000080008081,
        0x8000000000008009,
        0x000000000000008a,
        0x0000000000000088,
        0x0000000080008009,
        0x000000008000000a,
        0x000000008000808b,
        0x800000000000008b,
        0x8000000000008089,
        0x8000000000008003,
        0x8000000000008002,
        0x8000000000000080,
        0x000000000000800a,
        0x800000008000000a,
        0x8000000080008081,
        0x8000000000008080,
        0x0000000080000001,
        0x8000000080008008,
    ];
    const ROT: [u32; 24] = [
        1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
    ];
    const PI: [usize; 24] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
    ];
    for &rc in &RC {
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = s[x] ^ s[x + 5] ^ s[x + 10] ^ s[x + 15] ^ s[x + 20];
        }
        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
        }
        for x in 0..5 {
            for y in 0..5 {
                s[x + 5 * y] ^= d[x];
            }
        }
        let mut b = [0u64; 25];
        let mut t = s[1];
        for i in 0..24 {
            let j = PI[i];
            b[j] = t.rotate_left(ROT[i]);
            t = s[j];
        }

        b[0] = s[0];
        for y in 0..5 {
            for x in 0..5 {
                s[x + 5 * y] = b[x + 5 * y] ^ ((!b[(x + 1) % 5 + 5 * y]) & b[(x + 2) % 5 + 5 * y]);
            }
        }
        s[0] ^= rc;
    }
}
struct Sha3 {
    st: [u64; 25],
    rate: usize,
    buf: Vec<u8>,
    out: usize,
}
impl Sha3 {
    fn new(bits: usize) -> Self {
        Self {
            st: [0u64; 25],
            rate: (1600 - 2 * bits) / 8,
            buf: Vec::new(),
            out: bits / 8,
        }
    }
    fn absorb(&mut self, d: &[u8]) {
        self.buf.extend_from_slice(d);
        while self.buf.len() >= self.rate {
            let blk: Vec<u8> = self.buf.drain(..self.rate).collect();
            for i in 0..self.rate / 8 {
                self.st[i] ^= u64::from_le_bytes(blk[i * 8..i * 8 + 8].try_into().unwrap());
            }
            keccakf(&mut self.st);
        }
    }
    fn squeeze(mut self) -> Vec<u8> {
        let mut pad = vec![0u8; self.rate];
        let n = self.buf.len();
        pad[..n].copy_from_slice(&self.buf);
        pad[n] ^= 0x06;
        let l = self.rate - 1;
        pad[l] ^= 0x80;
        for i in 0..self.rate / 8 {
            self.st[i] ^= u64::from_le_bytes(pad[i * 8..i * 8 + 8].try_into().unwrap());
        }
        keccakf(&mut self.st);
        let mut o = Vec::new();
        while o.len() < self.out {
            for &w in &self.st {
                if o.len() >= self.out {
                    break;
                }
                let b = w.to_le_bytes();
                let k = (self.out - o.len()).min(8);
                o.extend_from_slice(&b[..k]);
            }
            if o.len() < self.out {
                keccakf(&mut self.st);
            }
        }
        o
    }
}
struct Sha3Box {
    inner: Sha3,
}
impl Upd for Sha3Box {
    fn upd(&mut self, d: &[u8]) {
        self.inner.absorb(d);
    }
    fn fin(&mut self) -> Vec<u8> {
        std::mem::replace(&mut self.inner, Sha3::new(512)).squeeze()
    }
}
applet!(
    Sha3sumApplet,
    "sha3sum",
    "Print or check SHA3 checksums",
    run_sha3sum
);
fn run_sha3sum(args: &[OsString]) -> Result<i32> {
    let mut bits = 512usize;
    let mut rest: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-a" {
            if i + 1 >= args.len() {
                eprintln!("sha3sum: -a needs an argument");
                return Ok(1);
            }
            bits = match ab(&args[i + 1]) {
                b"224" => 224,
                b"256" => 256,
                b"384" => 384,
                b"512" => 512,
                _ => {
                    eprintln!("sha3sum: unsupported -a value (want 224|256|384|512)");
                    return Ok(1);
                }
            };
            i += 2;
        } else if b.starts_with(b"-a") && b.len() > 2 {
            bits = match &b[2..] {
                b"224" => 224,
                b"256" => 256,
                b"384" => 384,
                b"512" => 512,
                _ => {
                    eprintln!("sha3sum: unsupported -a value");
                    return Ok(1);
                }
            };
            i += 1;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    sum_main("sha3sum", &rest, || {
        Box::new(Sha3Box {
            inner: Sha3::new(bits),
        })
    })
}

fn b64_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = Vec::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16
            | (c.get(1).copied().unwrap_or(0) as u32) << 8
            | (c.get(2).copied().unwrap_or(0) as u32);
        o.push(T[(n >> 18) as usize & 63]);
        o.push(T[(n >> 12) as usize & 63]);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63]
        } else {
            b'='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63]
        } else {
            b'='
        });
    }
    o
}
fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => Some(64),
        _ => None,
    }
}
fn b64_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        match b64_val(c) {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(4) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::with_capacity(digs.len() / 4 * 3);
    for c in digs.chunks(4) {
        let pad = c.iter().rev().take_while(|&&v| v == 64).count();
        if pad > 2 {
            return Err("invalid input".to_string());
        }
        let mut n = 0u32;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 64 { 0 } else { v as u32 };
            n |= v << (18 - 6 * i);
        }
        o.push((n >> 16) as u8);
        if pad < 2 {
            o.push((n >> 8) as u8);
        }
        if pad < 1 {
            o.push(n as u8);
        }
    }
    Ok(o)
}
fn b32_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut o = Vec::with_capacity(d.len().div_ceil(5) * 8);
    for c in d.chunks(5) {
        let mut n = 0u64;
        for &b in c {
            n = (n << 8) | b as u64;
        }
        n <<= (5 - c.len()) * 8;
        let nd = (c.len() * 8).div_ceil(5);
        for i in 0..8 {
            o.push(if i < nd {
                T[(n >> (35 - 5 * i)) as usize & 31]
            } else {
                b'='
            });
        }
    }
    o
}
fn b32_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        let v = match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26 - 24 + 24 - 24),
            b'2'..=b'7' => Some(c - b'2' + 26),
            b'=' => Some(32),
            _ => None,
        };

        let v = match c {
            b'a'..=b'z' => Some(c - b'a'),
            _ => v,
        };
        match v {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(8) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::new();
    for c in digs.chunks(8) {
        let pad = c.iter().rev().take_while(|&&v| v == 32).count();
        let mut n = 0u64;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 32 { 0 } else { v as u64 };
            n |= v << (35 - 5 * i);
        }
        let nb = 5 - pad * 5 / 8;
        for i in 0..nb {
            o.push((n >> (32 - 8 * i)) as u8);
        }
    }
    Ok(o)
}
fn bxx_main(name: &str, args: &[OsString], is32: bool) -> Result<i32> {
    let (mut dec, mut ign, mut wrap) = (false, false, 76usize);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" || b == b"--decode" {
            dec = true;
        } else if b == b"-i" || b == b"--ignore-garbage" {
            ign = true;
        } else if b == b"-w" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -w needs an argument", name);
                return Ok(1);
            }
            wrap = lossy(&args[i]).parse().unwrap_or(76);
        } else if b.starts_with(b"-w") && b.len() > 2 {
            wrap = String::from_utf8_lossy(&b[2..]).parse().unwrap_or(76);
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" && !b.starts_with(b"--") {
            let mut ok = true;
            for &c in &b[1..] {
                match c {
                    b'd' => dec = true,
                    b'i' => ign = true,
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                eprintln!("{}: invalid option '{}'", name, lossy(&args[i]));
                return Ok(1);
            }
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let data = match read_all(f.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}: can't open '{}': {}", name, lossy(f), e);
                rc = 1;
                continue;
            }
        };
        if dec {
            let r = if is32 {
                b32_dec(&data, ign)
            } else {
                b64_dec(&data, ign)
            };
            match r {
                Ok(v) => {
                    if out.write_all(&v).is_err() {
                        rc = 1;
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", name, e);
                    rc = 1;
                }
            }
        } else {
            let e = if is32 { b32_enc(&data) } else { b64_enc(&data) };
            if wrap == 0 {
                let _ = out.write_all(&e);
                let _ = out.write_all(b"\n");
            } else {
                for c in e.chunks(wrap) {
                    let _ = out.write_all(c);
                    let _ = out.write_all(b"\n");
                }
            }
        }
    }
    Ok(rc)
}
applet!(
    Base32Applet,
    "base32",
    "Base32 encode or decode",
    run_base32
);
fn run_base32(args: &[OsString]) -> Result<i32> {
    bxx_main("base32", args, true)
}
applet!(
    Base64Applet,
    "base64",
    "Base64 encode or decode",
    run_base64
);
fn run_base64(args: &[OsString]) -> Result<i32> {
    bxx_main("base64", args, false)
}

fn crc_tab() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (i, slot) in t.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    t
}
applet!(Crc32Applet, "crc32", "Print CRC32 checksums", run_crc32);
fn run_crc32(args: &[OsString]) -> Result<i32> {
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"--" {
            continue;
        }
        if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("crc32: invalid option '{}'", lossy(a));
            return Ok(1);
        }
        files.push(a.clone());
    }
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let tab = crc_tab();
    let mut rc = 0;
    let mut out = wlock();
    let mut line = Vec::with_capacity(64);
    for f in &files {
        let mut crc = 0xffff_ffffu32;
        let mut len = 0u64;
        let r = if f.as_os_str().as_bytes() == b"-" {
            let mut inp = std::io::stdin().lock();
            let mut buf = [0u8; 8192];
            loop {
                match inp.read(&mut buf) {
                    Ok(0) => break Ok(()),
                    Ok(n) => {
                        len += n as u64;
                        for &x in &buf[..n] {
                            crc = tab[((crc ^ x as u32) & 0xff) as usize] ^ (crc >> 8);
                        }
                        continue;
                    }
                    Err(e) => break Err(e),
                }
            }
        } else {
            match File::open(Path::new(f)) {
                Ok(mut fh) => {
                    let mut buf = [0u8; 8192];
                    loop {
                        match fh.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => {
                                len += n as u64;
                                for &x in &buf[..n] {
                                    crc = tab[((crc ^ x as u32) & 0xff) as usize] ^ (crc >> 8);
                                }
                                continue;
                            }
                            Err(e) => break Err(e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("crc32: can't open '{}': {}", lossy(f), e);
                    rc = 1;
                    continue;
                }
            }
        };
        if let Err(e) = r {
            eprintln!("crc32: {}: {}", lossy(f), e);
            rc = 1;
            continue;
        }
        crc ^= 0xffff_ffff;
        line.clear();
        let _ = writeln!(line, "{:08x}\t{}\t{}", crc, len, lossy(f));
        let _ = out.write_all(&line);
    }
    Ok(rc)
}

applet!(
    MakemimeApplet,
    "makemime",
    "Create MIME-encoded message",
    run_makemime
);
fn run_makemime(args: &[OsString]) -> Result<i32> {
    let (mut ctype, mut cdisp, mut eol) = (
        String::from("application/octet-stream"),
        None::<String>,
        "\n",
    );
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-c" {
            i += 1;
            if i >= args.len() {
                eprintln!("makemime: -c needs an argument");
                return Ok(1);
            }
            ctype = lossy(&args[i]);
        } else if b.starts_with(b"-c") && b.len() > 2 {
            ctype = String::from_utf8_lossy(&b[2..]).into_owned();
        } else if b == b"-C" {
            i += 1;
            if i >= args.len() {
                eprintln!("makemime: -C needs an argument");
                return Ok(1);
            }
            cdisp = Some(lossy(&args[i]));
        } else if b == b"-e" {
            eol = "\r\n";
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("makemime: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let inp: Vec<u8> = if files.is_empty() {
        let mut v = Vec::new();
        if std::io::stdin().lock().read_to_end(&mut v).is_err() {
            return Ok(1);
        }
        v
    } else {
        match read_all(files[0].as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("makemime: can't open '{}': {}", lossy(&files[0]), e);
                return Ok(1);
            }
        }
    };
    let mut out = wlock();
    let disp = cdisp.as_deref().unwrap_or("attachment");
    let _ = write!(out, "Content-Type: {}{}Content-Transfer-Encoding: base64{}Content-Disposition: {}; filename=\"{}\"{}", ctype, eol, eol, disp, files.first().map(lossy).unwrap_or_else(|| "stdin".to_string()), eol);
    let _ = write!(out, "{}", eol);
    for c in b64_enc(&inp).chunks(76) {
        let _ = out.write_all(c);
        let _ = out.write_all(eol.as_bytes());
    }
    Ok(0)
}
applet!(
    ReformimeApplet,
    "reformime",
    "Parse MIME-encoded message",
    run_reformime
);
fn run_reformime(args: &[OsString]) -> Result<i32> {
    let (mut idx, mut want_hdr, mut outfile): (Option<usize>, bool, Option<OsString>) =
        (None, false, None);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-i" {
            i += 1;
            if i >= args.len() {
                eprintln!("reformime: -i needs an argument");
                return Ok(1);
            }
            match lossy(&args[i]).parse() {
                Ok(n) => idx = Some(n),
                Err(_) => {
                    eprintln!("reformime: invalid index");
                    return Ok(1);
                }
            }
        } else if b.starts_with(b"-i") && b.len() > 2 {
            match String::from_utf8_lossy(&b[2..]).parse() {
                Ok(n) => idx = Some(n),
                Err(_) => {
                    eprintln!("reformime: invalid index");
                    return Ok(1);
                }
            }
        } else if b == b"-e" {
            want_hdr = true;
        } else if b == b"-o" {
            i += 1;
            if i >= args.len() {
                eprintln!("reformime: -o needs an argument");
                return Ok(1);
            }
            outfile = Some(args[i].clone());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("reformime: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let data = if files.is_empty() {
        match read_all(OsStr::from_bytes(b"-")) {
            Ok(v) => v,
            Err(_) => return Ok(1),
        }
    } else {
        match read_all(files[0].as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("reformime: can't open '{}': {}", lossy(&files[0]), e);
                return Ok(1);
            }
        }
    };

    let text = String::from_utf8_lossy(&data);
    let mut boundary: Option<String> = None;
    for line in text.lines() {
        let l = line.trim().to_ascii_lowercase();
        if let Some(p) = l.find("boundary=") {
            let mut v = line[p + 9..].trim().trim_matches('"').trim().to_string();
            v = v.trim_matches('"').to_string();
            if !v.is_empty() {
                boundary = Some(v);
            }
        }
    }
    let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
    if let Some(bnd) = boundary {
        let mark = format!("--{}", bnd);
        for part in text.split(&mark) {
            if part.contains("--") && part.trim_start().starts_with("--") {
                continue;
            }
            let (hdr, body) = match part.find("\n\n") {
                Some(p) => (part[..p].to_string(), part.as_bytes()[p + 2..].to_vec()),
                None => (String::new(), part.as_bytes().to_vec()),
            };
            if body
                .iter()
                .any(|&c| c != b'\r' && c != b'\n' && c != b' ' && c != b'\t' && c != b'-')
            {
                sections.push((hdr, body));
            }
        }
    } else {
        let (hdr, body) = match text.find("\n\n") {
            Some(p) => (text[..p].to_string(), text[p + 2..].as_bytes().to_vec()),
            None => (String::new(), data.clone()),
        };
        sections.push((hdr, body));
    }

    let mut out = wlock();
    let mut show = |n: usize| -> Result<i32> {
        if n >= sections.len() {
            eprintln!("reformime: no section {}", n);
            return Ok(1);
        }
        let (hdr, body) = &sections[n];
        let is_b64 = hdr.to_ascii_lowercase().contains("base64");
        let payload = if is_b64 {
            match b64_dec(body, true) {
                Ok(v) => v,
                Err(_) => body.clone(),
            }
        } else {
            body.clone()
        };
        if want_hdr {
            let _ = out.write_all(hdr.as_bytes());
            let _ = out.write_all(b"\n\n");
        }
        if let Some(o) = &outfile {
            match File::create(Path::new(o)) {
                Ok(mut f) => {
                    if f.write_all(&payload).is_err() {
                        return Ok(1);
                    }
                }
                Err(e) => {
                    eprintln!("reformime: can't write '{}': {}", lossy(o), e);
                    return Ok(1);
                }
            }
        } else {
            let _ = out.write_all(&payload);
        }
        Ok(0)
    };
    match idx {
        Some(n) => show(n),
        None => {
            for (n, (hdr, _)) in sections.iter().enumerate() {
                let ct = hdr
                    .lines()
                    .find(|l| l.to_ascii_lowercase().starts_with("content-type"))
                    .unwrap_or("content-type: text/plain");
                let _ = writeln!(out, "section {}: {}", n, ct);
            }
            Ok(0)
        }
    }
}

applet!(AsciiApplet, "ascii", "Print ASCII chart", run_ascii);
fn run_ascii(args: &[OsString]) -> Result<i32> {
    if !args.is_empty() {
        eprintln!("ascii: too many arguments");
        return Ok(1);
    }
    let names = [
        "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR",
        "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB",
        "ESC", "FS", "GS", "RS", "US", "SP",
    ];
    let mut out = wlock();
    let _ = writeln!(out, "Dec Hex Char Dec Hex Char Dec Hex Char Dec Hex Char");
    for r in 0u8..32 {
        for c in [r, r + 32, r + 64, r + 96] {
            let ch = if c < 33 {
                names[c as usize]
            } else if c == 127 {
                "DEL"
            } else {
                ""
            };
            if c < 33 || c == 127 {
                let _ = write!(out, "{:3} {:02X} {:<4}", c, c, ch);
            } else {
                let _ = write!(out, "{:3} {:02X} {}   ", c, c, c as char);
            }
            let _ = write!(out, "  ");
        }
        let _ = writeln!(out);
    }
    Ok(0)
}

fn dump_lines(out: &mut impl Write, data: &[u8], base: u64, cols: usize, up: bool) {
    let mut off = base;
    let mut line = Vec::with_capacity(128);
    for c in data.chunks(cols.max(1)) {
        line.clear();
        let _ = write!(line, "{:08x}  ", off);
        for &b in c {
            if up {
                let _ = write!(line, "{:02X} ", b);
            } else {
                line.extend_from_slice(&[
                    b"0123456789abcdef"[(b >> 4) as usize],
                    b"0123456789abcdef"[(b & 15) as usize],
                    b' ',
                ]);
            }
        }

        let _ = write!(line, " |");
        for &b in c {
            line.push(if (32..127).contains(&b) { b } else { b'.' });
        }
        line.push(b'|');
        line.push(b'\n');
        let _ = out.write_all(&line);
        off += c.len() as u64;
    }
}
fn hd_main(name: &str, args: &[OsString], canonical_default: bool) -> Result<i32> {
    let (mut cols, mut off, mut len, mut verb) = (16usize, 0u64, u64::MAX, false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-v" {
            verb = true;
        } else if b == b"-C" {
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -n needs an argument", name);
                return Ok(1);
            }
            len = lossy(&args[i]).parse().unwrap_or(u64::MAX);
        } else if b.starts_with(b"-n") && b.len() > 2 {
            len = String::from_utf8_lossy(&b[2..]).parse().unwrap_or(u64::MAX);
        } else if b == b"-s" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -s needs an argument", name);
                return Ok(1);
            }
            let s = lossy(&args[i]);
            off = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).unwrap_or(0)
            } else {
                s.parse().unwrap_or(0)
            };
        } else if b.starts_with(b"-s") && b.len() > 2 {
            let s = String::from_utf8_lossy(&b[2..]).into_owned();
            off = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).unwrap_or(0)
            } else {
                s.parse().unwrap_or(0)
            };
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("{}: invalid option '{}'", name, lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        let _ = (canonical_default, verb, cols);
        i += 1;
    }
    let _ = &mut cols;
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let data = match read_all(f.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}: can't open '{}': {}", name, lossy(f), e);
                rc = 1;
                continue;
            }
        };
        let start = (off as usize).min(data.len());
        let end = (start as u64 + len).min(data.len() as u64) as usize;
        dump_lines(&mut out, &data[start..end], off, 16, false);
        let _ = writeln!(out, "{:08x}", off + (end - start) as u64);
    }
    Ok(rc)
}
applet!(HdApplet, "hd", "Hexdump in canonical form", run_hd);
fn run_hd(args: &[OsString]) -> Result<i32> {
    hd_main("hd", args, true)
}
applet!(HexdumpApplet, "hexdump", "Dump file in hex", run_hexdump);
fn run_hexdump(args: &[OsString]) -> Result<i32> {
    hd_main("hexdump", args, true)
}
applet!(
    HexeditApplet,
    "hexedit",
    "Show or patch bytes at offsets",
    run_hexedit
);
fn run_hexedit(args: &[OsString]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("hexedit: needs a file argument");
        return Ok(1);
    }
    let path = Path::new(&args[0]).to_path_buf();
    if args.len() == 1 {
        let data = match read_all(path.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("hexedit: can't open '{}': {}", lossy(&args[0]), e);
                return Ok(1);
            }
        };
        let mut out = wlock();
        dump_lines(&mut out, &data, 0, 16, false);
        let _ = writeln!(out, "{:08x}", data.len());
        return Ok(0);
    }
    let off_s = lossy(&args[1]);
    let off: u64 = if let Some(h) = off_s
        .strip_prefix("0x")
        .or_else(|| off_s.strip_prefix("0X"))
    {
        match u64::from_str_radix(h, 16) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("hexedit: bad offset");
                return Ok(1);
            }
        }
    } else {
        match off_s.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("hexedit: bad offset");
                return Ok(1);
            }
        }
    };
    let mut bytes: Vec<u8> = Vec::new();
    for a in &args[2..] {
        let s = lossy(a);
        let t = s.trim();
        if !t.len().is_multiple_of(2) {
            eprintln!("hexedit: bad hex byte '{}'", t);
            return Ok(1);
        }
        for i in (0..t.len()).step_by(2) {
            match u8::from_str_radix(&t[i..i + 2], 16) {
                Ok(v) => bytes.push(v),
                Err(_) => {
                    eprintln!("hexedit: bad hex byte '{}'", t);
                    return Ok(1);
                }
            }
        }
    }
    use std::io::{Seek, SeekFrom};
    match File::options().read(true).write(true).open(&path) {
        Ok(mut f) => {
            if f.seek(SeekFrom::Start(off)).is_err() || f.write_all(&bytes).is_err() {
                eprintln!("hexedit: write failed");
                return Ok(1);
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("hexedit: can't open '{}': {}", lossy(&args[0]), e);
            Ok(1)
        }
    }
}

struct VolInfo {
    label: String,
    uuid: String,
    fstype: String,
}
fn probe_vol(path: &OsStr) -> Option<VolInfo> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(Path::new(path)).ok()?;

    let mut boot = [0u8; 512];
    if f.read_exact(&mut boot).is_ok() && boot[510] == 0x55 && boot[511] == 0xaa {
        let is_fat32 = boot[82..90] == *b"FAT32   ";
        let is_fat = is_fat32 || boot[54..62] == *b"FAT12   " || boot[54..62] == *b"FAT16   ";
        if is_fat {
            let (lab_off, ser_off) = if is_fat32 { (71, 67) } else { (43, 39) };
            let label = String::from_utf8_lossy(&boot[lab_off..lab_off + 11])
                .trim()
                .to_string();
            let ser = u32::from_le_bytes(boot[ser_off..ser_off + 4].try_into().unwrap_or([0; 4]));
            return Some(VolInfo {
                label,
                uuid: format!("{:04X}-{:04X}", ser >> 16, ser & 0xffff),
                fstype: "vfat".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut sb = [0u8; 256];
        if f.read_exact(&mut sb).is_ok() && sb[56] == 0x53 && sb[57] == 0xef {
            let label = String::from_utf8_lossy(&sb[120..136])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &sb[104..120];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "ext2/3/4".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut pg = vec![0u8; 4096];
        if f.read_exact(&mut pg).is_ok() && pg[4096 - 10..] == *b"SWAPSPACE2" {
            let label = String::from_utf8_lossy(&pg[1052..1068])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &pg[1036..1052];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "swap".to_string(),
            });
        }
    }
    None
}

fn candidate_devs() -> Vec<OsString> {
    let mut v: Vec<OsString> = Vec::new();
    if let Ok(m) = std::fs::read("/proc/mounts") {
        for line in m.split(|&c| c == b'\n') {
            if let Some(sp) = line.iter().position(|&c| c == b' ') {
                let src = &line[..sp];
                if src.starts_with(b"/dev/") {
                    v.push(OsString::from(OsStr::from_bytes(src)));
                }
            }
        }
    }
    if let Ok(sys) = std::fs::read_dir("/sys/block") {
        for e in sys.flatten() {
            let nm = e.file_name();
            let devp = format!("/dev/{}", nm.to_string_lossy());
            let o = OsString::from(&devp);
            if !v.contains(&o) {
                v.push(o);
            }
        }
    }
    for guess in [
        "/dev/sda1",
        "/dev/vda1",
        "/dev/nvme0n1p1",
        "/dev/mmcblk0p1",
        "/dev/dm-0",
    ] {
        let o = OsString::from(guess);
        if !v.contains(&o) {
            v.push(o);
        }
    }
    v
}
applet!(VolnameApplet, "volname", "Print volume label", run_volname);
fn run_volname(args: &[OsString]) -> Result<i32> {
    if args.len() != 1 {
        eprintln!("volname: needs exactly one device argument");
        return Ok(1);
    }
    match probe_vol(args[0].as_os_str()) {
        Some(vi) if !vi.label.is_empty() => {
            println!("{}", vi.label);
            Ok(0)
        }
        Some(_) => {
            eprintln!("volname: no label on '{}'", lossy(&args[0]));
            Ok(1)
        }
        None => {
            eprintln!("volname: can't read '{}'", lossy(&args[0]));
            Ok(1)
        }
    }
}
applet!(
    BlkidApplet,
    "blkid",
    "Locate/print block device attributes",
    run_blkid
);
fn run_blkid(args: &[OsString]) -> Result<i32> {
    let (mut tag, mut out_fmt, mut match_t) = (None::<String>, None::<String>, None::<String>);
    let mut devs: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-s" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -s needs an argument");
                return Ok(1);
            }
            tag = Some(lossy(&args[i]));
        } else if b.starts_with(b"-s") && b.len() > 2 {
            tag = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-o" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -o needs an argument");
                return Ok(1);
            }
            out_fmt = Some(lossy(&args[i]));
        } else if b.starts_with(b"-o") && b.len() > 2 {
            out_fmt = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-t" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -t needs an argument");
                return Ok(1);
            }
            match_t = Some(lossy(&args[i]));
        } else if b.starts_with(b"-t") && b.len() > 2 {
            match_t = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-c" {
            i += 1;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("blkid: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            devs.push(args[i].clone());
        }
        i += 1;
    }
    if devs.is_empty() {
        devs = candidate_devs();
    }
    let val_only = out_fmt.as_deref() == Some("value");
    let mut rc = 0;
    let mut found = false;
    let mut out = wlock();
    for d in &devs {
        let vi = match probe_vol(d.as_os_str()) {
            Some(v) => v,
            None => continue,
        };
        if let Some(m) = &match_t {
            let (k, vv) = match m.split_once('=') {
                Some(x) => x,
                None => continue,
            };
            let ok = match k {
                "LABEL" => vi.label == vv,
                "UUID" => vi.uuid.eq_ignore_ascii_case(vv),
                "TYPE" => vi.fstype == vv,
                _ => false,
            };
            if !ok {
                continue;
            }
        }
        found = true;
        if val_only {
            let s = match tag.as_deref() {
                Some("LABEL") => vi.label.clone(),
                Some("UUID") => vi.uuid.clone(),
                Some("TYPE") => vi.fstype.clone(),
                _ => vi.uuid.clone(),
            };
            let _ = writeln!(out, "{}", s);
        } else if let Some(t) = &tag {
            let s = match t.as_str() {
                "LABEL" => vi.label.clone(),
                "UUID" => vi.uuid.clone(),
                "TYPE" => vi.fstype.clone(),
                _ => String::new(),
            };
            let _ = writeln!(out, "{}: {}=\"{}\"", lossy(d), t, s);
        } else {
            let _ = writeln!(
                out,
                "{}: LABEL=\"{}\" UUID=\"{}\" TYPE=\"{}\"",
                lossy(d),
                vi.label,
                vi.uuid,
                vi.fstype
            );
        }
    }
    if match_t.is_some() && !found {
        rc = 2;
    }
    Ok(rc)
}
applet!(
    FindfsApplet,
    "findfs",
    "Find filesystem by label or UUID",
    run_findfs
);
fn run_findfs(args: &[OsString]) -> Result<i32> {
    if args.len() != 1 {
        eprintln!("findfs: needs exactly one LABEL=/UUID= argument");
        return Ok(1);
    }
    let q = lossy(&args[0]);
    let (k, vv) = match q.split_once('=') {
        Some(x) => x,
        None => {
            eprintln!("findfs: argument must be LABEL=<label> or UUID=<uuid>");
            return Ok(1);
        }
    };
    if k != "LABEL" && k != "UUID" {
        eprintln!("findfs: argument must be LABEL=<label> or UUID=<uuid>");
        return Ok(1);
    }
    for d in candidate_devs() {
        if let Some(vi) = probe_vol(d.as_os_str()) {
            let hit = if k == "LABEL" {
                vi.label == vv
            } else {
                vi.uuid.eq_ignore_ascii_case(vv)
            };
            if hit {
                println!("{}", lossy(&d));
                return Ok(0);
            }
        }
    }
    eprintln!("findfs: unable to resolve '{}'", q);
    Ok(1)
}

const F_GET: libc::c_ulong = 0x80086601;
const F_SET: libc::c_ulong = 0x40086602;
const ATTR_BITS: [(u8, libc::c_long); 13] = [
    (b's', 0x0000_0001),
    (b'u', 0x0000_0002),
    (b'c', 0x0000_0004),
    (b'S', 0x0000_0008),
    (b'i', 0x0000_0010),
    (b'a', 0x0000_0020),
    (b'A', 0x0000_0080),
    (b'd', 0x0000_0040),
    (b'D', 0x0001_0000),
    (b'E', 0x0000_0800),
    (b'e', 0x0008_0000),
    (b'I', 0x0000_1000),
    (b'j', 0x0000_4000),
];
const ATTR_ORDER: [u8; 13] = *b"sucSiaAdDEeIj";
fn bit_of(c: u8) -> Option<libc::c_long> {
    ATTR_BITS.iter().find(|&&(x, _)| x == c).map(|&(_, b)| b)
}
fn get_flags(p: &OsStr) -> std::io::Result<libc::c_long> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut fl: libc::c_long = 0;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_GET as _, &mut fl) };
    let _ = f;
    let _ = OsStr::from_bytes;
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(fl)
    }
}
fn set_flags(p: &OsStr, fl: libc::c_long) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let f = File::open(Path::new(p))?;
    let mut flm = fl;
    let r = unsafe { libc::ioctl(f.as_raw_fd(), F_SET as _, &mut flm) };
    if r < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn flags_str(fl: libc::c_long) -> String {
    let mut s = String::with_capacity(20);
    for &c in &ATTR_ORDER {
        let b = bit_of(c).unwrap_or(0);
        s.push(if fl & b != 0 { c as char } else { '-' });
    }
    s
}
applet!(LsattrApplet, "lsattr", "List file attributes", run_lsattr);
fn run_lsattr(args: &[OsString]) -> Result<i32> {
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"--" {
            continue;
        }
        if b.len() > 1 && b[0] == b'-' && b != b"-" {
            let mut ok = true;
            for &c in &b[1..] {
                if !matches!(c, b'a' | b'd' | b'R' | b'v') {
                    ok = false;
                    break;
                }
            }
            if !ok {
                eprintln!("lsattr: invalid option '{}'", lossy(a));
                return Ok(1);
            }
            continue;
        }
        files.push(a.clone());
    }
    if files.is_empty() {
        eprintln!("lsattr: needs a file argument");
        return Ok(1);
    }
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        match get_flags(f.as_os_str()) {
            Ok(fl) => {
                let _ = writeln!(out, "{} {}", flags_str(fl), lossy(f));
            }
            Err(e) => {
                eprintln!("lsattr: can't read flags of '{}': {}", lossy(f), e);
                rc = 1;
            }
        }
    }
    Ok(rc)
}
applet!(ChattrApplet, "chattr", "Change file attributes", run_chattr);
fn run_chattr(args: &[OsString]) -> Result<i32> {
    let (mut op_arg, mut recurse) = (None::<(u8, Vec<u8>)>, false);
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-R" {
            recurse = true;
        } else if b == b"-V" || b == b"-f" {
        } else if b.len() > 1 && (b[1] == b'+' || b[1] == b'-' || b[1] == b'=') && b[0] == b'-' {
            eprintln!("chattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else if !b.is_empty() && (b[0] == b'+' || b[0] == b'-' || b[0] == b'=') {
            if op_arg.is_some() {
                eprintln!("chattr: only one operator allowed");
                return Ok(1);
            }
            let mut letters = Vec::new();
            for &c in &b[1..] {
                if bit_of(c).is_none() {
                    eprintln!("chattr: invalid attribute '{}'", c as char);
                    return Ok(1);
                }
                letters.push(c);
            }
            op_arg = Some((b[0], letters));
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("chattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            files.push(a.clone());
        }
    }
    let (op, letters) = match op_arg {
        Some(x) => x,
        None => {
            eprintln!("chattr: needs an operator (+/-/=) argument");
            return Ok(1);
        }
    };
    if files.is_empty() {
        eprintln!("chattr: needs a file argument");
        return Ok(1);
    }

    let mut targets = files.clone();
    if recurse {
        let mut extra = Vec::new();
        for f in &files {
            if let Ok(rd) = std::fs::read_dir(Path::new(f)) {
                for e in rd.flatten() {
                    extra.push(e.path().into_os_string());
                }
            }
        }
        targets.extend(extra);
    }
    let mut rc = 0;
    for t in &targets {
        let cur = match get_flags(t.as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("chattr: can't read flags of '{}': {}", lossy(t), e);
                rc = 1;
                continue;
            }
        };
        let mut mask = 0;
        for &c in &letters {
            mask |= bit_of(c).unwrap_or(0);
        }
        let new = match op {
            b'+' => cur | mask,
            b'-' => cur & !mask,
            _ => mask,
        };
        if let Err(e) = set_flags(t.as_os_str(), new) {
            eprintln!("chattr: can't set flags of '{}': {}", lossy(t), e);
            rc = 1;
        }
    }
    Ok(rc)
}

fn xenc(data: &[u8], fmt: &str) -> String {
    match fmt {
        "hex" => {
            let mut s = String::from("0x");
            for &b in data {
                use std::fmt::Write as _;
                let _ = write!(s, "{:02x}", b);
            }
            s
        }
        "base64" => String::from_utf8(b64_enc(data)).unwrap_or_default(),
        _ => String::from_utf8_lossy(data).into_owned(),
    }
}
applet!(
    GetfattrApplet,
    "getfattr",
    "Get extended attributes",
    run_getfattr
);
fn run_getfattr(args: &[OsString]) -> Result<i32> {
    let (mut name, mut dump, mut enc, mut no_der) =
        (None::<String>, false, String::from("text"), false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" || b == b"--dump" {
            dump = true;
        } else if b == b"-h" {
            no_der = true;
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("getfattr: -n needs an argument");
                return Ok(1);
            }
            name = Some(lossy(&args[i]));
        } else if b.starts_with(b"-n") && b.len() > 2 {
            name = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-e" {
            i += 1;
            if i >= args.len() {
                eprintln!("getfattr: -e needs an argument");
                return Ok(1);
            }
            enc = lossy(&args[i]);
        } else if b.starts_with(b"-e") && b.len() > 2 {
            enc = String::from_utf8_lossy(&b[2..]).into_owned();
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("getfattr: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    if files.is_empty() || (name.is_none() && !dump) {
        eprintln!("getfattr: needs -n NAME or -d plus a file argument");
        return Ok(1);
    }
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let cp = match CString::new(f.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => {
                rc = 1;
                continue;
            }
        };
        let names: Vec<String> = if dump {
            let mut sz = unsafe { libc::listxattr(cp.as_ptr(), std::ptr::null_mut(), 0) };
            if no_der {
                sz = unsafe { libc::llistxattr(cp.as_ptr(), std::ptr::null_mut(), 0) };
            }
            if sz < 0 {
                eprintln!("getfattr: can't list '{}'", lossy(f));
                rc = 1;
                continue;
            }
            let mut buf = vec![0u8; sz as usize];
            let r = if no_der {
                unsafe { libc::llistxattr(cp.as_ptr(), buf.as_mut_ptr() as _, buf.len()) }
            } else {
                unsafe { libc::listxattr(cp.as_ptr(), buf.as_mut_ptr() as _, buf.len()) }
            };
            if r < 0 {
                eprintln!("getfattr: can't list '{}'", lossy(f));
                rc = 1;
                continue;
            }
            buf[..r as usize]
                .split(|&c| c == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect()
        } else {
            vec![name.clone().unwrap_or_default()]
        };
        for nm in &names {
            let cn = match CString::new(nm.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut sz =
                unsafe { libc::getxattr(cp.as_ptr(), cn.as_ptr(), std::ptr::null_mut(), 0) };
            if no_der {
                sz = unsafe { libc::lgetxattr(cp.as_ptr(), cn.as_ptr(), std::ptr::null_mut(), 0) };
            }
            if sz < 0 {
                eprintln!("getfattr: '{}' has no '{}'", lossy(f), nm);
                rc = 1;
                continue;
            }
            let mut buf = vec![0u8; sz as usize];
            let r = if no_der {
                unsafe {
                    libc::lgetxattr(cp.as_ptr(), cn.as_ptr(), buf.as_mut_ptr() as _, buf.len())
                }
            } else {
                unsafe {
                    libc::getxattr(cp.as_ptr(), cn.as_ptr(), buf.as_mut_ptr() as _, buf.len())
                }
            };
            if r < 0 {
                rc = 1;
                continue;
            }
            let _ = writeln!(out, "{}=\"{}\"", nm, xenc(&buf[..r as usize], &enc));
        }
    }
    Ok(rc)
}
applet!(
    SetfattrApplet,
    "setfattr",
    "Set extended attributes",
    run_setfattr
);
fn run_setfattr(args: &[OsString]) -> Result<i32> {
    let (mut name, mut val, mut remove, mut no_der) =
        (None::<Vec<u8>>, None::<Vec<u8>>, false, false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-h" {
            no_der = true;
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -n needs an argument");
                return Ok(1);
            }
            name = Some(ab(&args[i]).to_vec());
        } else if b.starts_with(b"-n") && b.len() > 2 {
            name = Some(b[2..].to_vec());
        } else if b == b"-v" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -v needs an argument");
                return Ok(1);
            }
            val = Some(ab(&args[i]).to_vec());
        } else if b.starts_with(b"-v") && b.len() > 2 {
            val = Some(b[2..].to_vec());
        } else if b == b"-x" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -x needs an argument");
                return Ok(1);
            }
            name = Some(ab(&args[i]).to_vec());
            remove = true;
        } else if b.starts_with(b"-x") && b.len() > 2 {
            name = Some(b[2..].to_vec());
            remove = true;
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("setfattr: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let nm = match name {
        Some(n) => n,
        None => {
            eprintln!("setfattr: needs -n NAME (or -x NAME)");
            return Ok(1);
        }
    };
    if !remove && val.is_none() {
        eprintln!("setfattr: needs -v VALUE");
        return Ok(1);
    }
    if files.is_empty() {
        eprintln!("setfattr: needs a file argument");
        return Ok(1);
    }
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let cn = match CString::new(nm) {
        Ok(c) => c,
        Err(_) => return Ok(1),
    };
    let mut rc = 0;
    for f in &files {
        let cp = match CString::new(f.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => {
                rc = 1;
                continue;
            }
        };
        let r = if remove {
            if no_der {
                unsafe { libc::lremovexattr(cp.as_ptr(), cn.as_ptr()) }
            } else {
                unsafe { libc::removexattr(cp.as_ptr(), cn.as_ptr()) }
            }
        } else {
            let vv = val.clone().unwrap_or_default();
            if no_der {
                unsafe { libc::lsetxattr(cp.as_ptr(), cn.as_ptr(), vv.as_ptr() as _, vv.len(), 0) }
            } else {
                unsafe { libc::setxattr(cp.as_ptr(), cn.as_ptr(), vv.as_ptr() as _, vv.len(), 0) }
            }
        };
        if r < 0 {
            eprintln!(
                "setfattr: can't set '{}': {}",
                lossy(f),
                std::io::Error::last_os_error()
            );
            rc = 1;
        }
    }
    Ok(rc)
}

applet!(
    FatattrApplet,
    "fatattr",
    "Show or change FAT attributes",
    run_fatattr
);
fn run_fatattr(args: &[OsString]) -> Result<i32> {
    const G: libc::c_ulong = 0x8004_7211;
    const S: libc::c_ulong = 0x4004_7211;
    let mut op: Vec<(u8, bool)> = Vec::new();
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b.len() > 1
            && (b[0] == b'+' || b[0] == b'-')
            && b[1..]
                .iter()
                .all(|&c| matches!(c, b'r' | b'h' | b's' | b'v' | b'd' | b'a'))
            && b.len() <= 7
        {
            let add = b[0] == b'+';
            for &c in &b[1..] {
                op.push((c, add));
            }
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("fatattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            files.push(a.clone());
        }
    }
    if files.is_empty() {
        eprintln!("fatattr: needs a file argument");
        return Ok(1);
    }
    use std::os::unix::io::AsRawFd;
    let bit = |c: u8| -> u32 {
        match c {
            b'r' => 0x01,
            b'h' => 0x02,
            b's' => 0x04,
            b'v' => 0x08,
            b'd' => 0x10,
            b'a' => 0x20,
            _ => 0,
        }
    };
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let fh = match File::open(Path::new(f)) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("fatattr: can't open '{}': {}", lossy(f), e);
                rc = 1;
                continue;
            }
        };
        let mut at: u32 = 0;
        if unsafe { libc::ioctl(fh.as_raw_fd(), G as _, &mut at) } < 0 {
            eprintln!(
                "fatattr: can't get attrs of '{}': {}",
                lossy(f),
                std::io::Error::last_os_error()
            );
            rc = 1;
            continue;
        }
        if op.is_empty() {
            let mut s = String::new();
            for c in *b"rhsvda" {
                s.push(if at & bit(c) != 0 { c as char } else { '-' });
            }
            let _ = writeln!(out, "{} {}", s, lossy(f));
        } else {
            for &(c, add) in &op {
                if add {
                    at |= bit(c);
                } else {
                    at &= !bit(c);
                }
            }
            if unsafe { libc::ioctl(fh.as_raw_fd(), S as _, &at) } < 0 {
                eprintln!(
                    "fatattr: can't set attrs of '{}': {}",
                    lossy(f),
                    std::io::Error::last_os_error()
                );
                rc = 1;
            }
        }
    }
    Ok(rc)
}

applet!(LosetupApplet, "losetup", "Set up loop devices", run_losetup);
fn run_losetup(args: &[OsString]) -> Result<i32> {
    let (mut show_all, mut find_free, mut detach) = (false, false, false);
    let mut rest: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-a" || b == b"--list" {
            show_all = true;
        } else if b == b"-f" || b == b"--find" {
            find_free = true;
        } else if b == b"-d" || b == b"--detach" {
            detach = true;
        } else if b == b"-o" || b == b"--offset" {
            eprintln!("losetup: -o/--offset is not supported in this build");
            return Ok(1);
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("losetup: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            rest.push(a.clone());
        }
    }
    use std::os::unix::io::AsRawFd;
    if show_all || (rest.is_empty() && !find_free && !detach) {
        let mut out = wlock();
        let mut n = 0u32;
        loop {
            let p = format!("/dev/loop{}", n);
            if Path::new(&p).exists() || Path::new(&format!("/sys/block/loop{}", n)).exists() {
                let bf = std::fs::read_to_string(format!("/sys/block/loop{}/loop/backing_file", n))
                    .unwrap_or_default();
                let bf = bf.trim();
                if !bf.is_empty() {
                    let _ = writeln!(out, "{}: {}", p, bf);
                }
                n += 1;
                if n > 256 {
                    break;
                }
            } else if n > 8 {
                break;
            } else {
                n += 1;
            }
        }
        return Ok(0);
    }
    if find_free && rest.is_empty() {
        let mut n = 0u32;
        loop {
            let p = format!("/dev/loop{}", n);
            let bf = std::fs::read_to_string(format!("/sys/block/loop{}/loop/backing_file", n))
                .unwrap_or_default();
            if bf.trim().is_empty()
                && (Path::new(&p).exists() || Path::new(&format!("/sys/block/loop{}", n)).exists())
            {
                println!("{}", p);
                return Ok(0);
            }
            n += 1;
            if n > 256 {
                eprintln!("losetup: no free loop device");
                return Ok(1);
            }
        }
    }
    if detach {
        if rest.is_empty() {
            eprintln!("losetup: -d needs a device");
            return Ok(1);
        }
        let mut rc = 0;
        for d in &rest {
            match File::options().read(true).write(true).open(Path::new(d)) {
                Ok(f) => {
                    if unsafe { libc::ioctl(f.as_raw_fd(), 0x4c01u64 as _) } < 0 {
                        eprintln!(
                            "losetup: detach '{}' failed: {}",
                            lossy(d),
                            std::io::Error::last_os_error()
                        );
                        rc = 1;
                    }
                }
                Err(e) => {
                    eprintln!("losetup: can't open '{}': {}", lossy(d), e);
                    rc = 1;
                }
            }
        }
        return Ok(rc);
    }

    if rest.len() != 2 {
        eprintln!("losetup: usage: losetup [-a|-f|-d DEV] [DEV FILE]");
        return Ok(1);
    }
    let (dev, file) = (&rest[0], &rest[1]);
    let lf = match File::options().read(true).write(true).open(Path::new(dev)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("losetup: can't open '{}': {}", lossy(dev), e);
            return Ok(1);
        }
    };
    let bf = match File::open(Path::new(file)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("losetup: can't open '{}': {}", lossy(file), e);
            return Ok(1);
        }
    };
    if unsafe { libc::ioctl(lf.as_raw_fd(), 0x4c00u64 as _, bf.as_raw_fd()) } < 0 {
        eprintln!(
            "losetup: attach failed: {}",
            std::io::Error::last_os_error()
        );
        return Ok(1);
    }
    Ok(0)
}
applet!(
    MountpointApplet,
    "mountpoint",
    "Check if directory is a mountpoint",
    run_mountpoint
);
fn run_mountpoint(args: &[OsString]) -> Result<i32> {
    let (mut quiet, mut print_dev) = (false, false);
    let mut dirs: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-q" || b == b"--quiet" {
            quiet = true;
        } else if b == b"-d" || b == b"-x" {
            print_dev = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("mountpoint: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            dirs.push(a.clone());
        }
    }
    if dirs.len() != 1 {
        eprintln!("mountpoint: needs exactly one directory");
        return Ok(1);
    }
    let p = Path::new(&dirs[0]);
    let st = match std::fs::metadata(p) {
        Ok(_) => unsafe {
            let mut s: libc::stat = std::mem::zeroed();
            let c = std::ffi::CString::new(p.as_os_str().as_bytes())
                .unwrap_or_else(|_| std::ffi::CString::new("/").unwrap());
            if libc::stat(c.as_ptr(), &mut s) != 0 {
                eprintln!("mountpoint: can't stat '{}'", lossy(&dirs[0]));
                return Ok(1);
            }
            s
        },
        Err(e) => {
            eprintln!("mountpoint: can't stat '{}': {}", lossy(&dirs[0]), e);
            return Ok(1);
        }
    };

    let canon = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let mut is_mp = false;
    if let Ok(m) = std::fs::read("/proc/mounts") {
        for line in m.split(|&c| c == b'\n') {
            let f: Vec<&[u8]> = line.split(|&c| c == b' ').collect();
            if f.len() >= 2 {
                let mp = Path::new(OsStr::from_bytes(f[1]));
                let mc = std::fs::canonicalize(mp).unwrap_or_else(|_| mp.to_path_buf());
                if mc == canon {
                    is_mp = true;
                    break;
                }
            }
        }
    }

    if !is_mp {
        if let Some(par) = p.parent() {
            let pp = if par.as_os_str().is_empty() {
                Path::new("/")
            } else {
                par
            };
            unsafe {
                let c = std::ffi::CString::new(pp.as_os_str().as_bytes())
                    .unwrap_or_else(|_| std::ffi::CString::new("/").unwrap());
                let mut ps: libc::stat = std::mem::zeroed();
                if libc::stat(c.as_ptr(), &mut ps) == 0 && ps.st_dev != st.st_dev {
                    is_mp = true;
                }
            }
        } else {
            is_mp = true;
        }
    }
    if print_dev {
        println!("{}:{}", libc::major(st.st_dev), libc::minor(st.st_dev));
    } else if !quiet {
        println!(
            "{} is {}a mountpoint",
            lossy(&dirs[0]),
            if is_mp { "" } else { "not " }
        );
    }
    if is_mp {
        Ok(0)
    } else {
        Ok(1)
    }
}

applet!(MdevApplet, "mdev", "Coldplug device helper", run_mdev);
fn run_mdev(args: &[OsString]) -> Result<i32> {
    let mut scan = false;
    for a in args {
        let b = ab(a);
        if b == b"-s" {
            scan = true;
        } else {
            eprintln!("mdev: invalid option '{}'", lossy(a));
            return Ok(1);
        }
    }
    if scan {
        let mut rc = 0;
        let mut stack = vec![
            String::from("/sys/devices"),
            String::from("/sys/class"),
            String::from("/sys/block"),
        ];
        let mut count = 0u32;
        while let Some(dir) = stack.pop() {
            let rd = match std::fs::read_dir(&dir) {
                Ok(r) => r,
                Err(_) => continue,
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() && !p.is_symlink() {
                    stack.push(p.to_string_lossy().into_owned());
                } else if p.file_name().is_some_and(|n| n == "uevent") {
                    if std::fs::write(&p, b"add").is_err() {
                        rc = 1;
                    } else {
                        count += 1;
                    }
                }
            }
            if count > 20000 {
                break;
            }
        }

        for sys in ["/sys/bus", "/sys/subsystem"] {
            if Path::new(sys).exists() {
                let _ = std::fs::write(format!("{}/uevent", sys), b"add");
            }
        }
        return Ok(rc);
    }

    let act = std::env::var("ACTION").unwrap_or_default();
    let devpath = std::env::var("DEVPATH").unwrap_or_default();
    if devpath.is_empty() {
        eprintln!("mdev: no DEVPATH in environment (run with -s or as hotplug helper)");
        return Ok(1);
    }
    if act == "remove" {
        let devname = std::env::var("DEVNAME").unwrap_or_default();
        if !devname.is_empty() {
            let _ = std::fs::remove_file(format!("/dev/{}", devname));
        }
        return Ok(0);
    }

    let uev = std::fs::read_to_string(format!("/sys{}/uevent", devpath)).unwrap_or_default();
    let mut major: Option<u32> = None;
    let mut minor: Option<u32> = None;
    let mut mode = 0o660u32;
    for line in uev.lines() {
        if let Some(v) = line.strip_prefix("MAJOR=") {
            major = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("MINOR=") {
            minor = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("DEVMODE=") {
            mode = u32::from_str_radix(v.trim().trim_start_matches("0o"), 8).unwrap_or(0o660);
        }
    }
    let devname = std::env::var("DEVNAME")
        .unwrap_or_else(|_| devpath.rsplit('/').next().unwrap_or("node").to_string());
    let (maj, min) = match (major, minor) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("mdev: no MAJOR/MINOR for {}", devpath);
            return Ok(1);
        }
    };
    let is_block = uev.contains("DEVTYPE=disk")
        || uev.contains("DEVTYPE=partition")
        || std::env::var("SUBSYSTEM").unwrap_or_default() == "block";
    let ftype = if is_block {
        libc::S_IFBLK
    } else {
        libc::S_IFCHR
    };
    let dst = format!("/dev/{}", devname);
    let c = match std::ffi::CString::new(dst.as_bytes()) {
        Ok(c) => c,
        Err(_) => return Ok(1),
    };
    let dev = libc::makedev(maj, min);
    if unsafe { libc::mknod(c.as_ptr(), ftype | mode, dev) } != 0 {
        let e = std::io::Error::last_os_error();
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("mdev: mknod '{}' failed: {}", dst, e);
            return Ok(1);
        }
    }
    let _ = std::fs::write(format!("/sys{}/uevent", devpath), b"add");
    Ok(0)
}
applet!(UeventApplet, "uevent", "Print kernel uevents", run_uevent);
fn run_uevent(args: &[OsString]) -> Result<i32> {
    let mut count: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-c" {
            i += 1;
            if i >= args.len() {
                eprintln!("uevent: -c needs an argument");
                return Ok(1);
            }
            match lossy(&args[i]).parse() {
                Ok(n) => count = Some(n),
                Err(_) => {
                    eprintln!("uevent: bad count");
                    return Ok(1);
                }
            }
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("uevent: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        }
        i += 1;
    }
    unsafe {
        let fd = libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_DGRAM | libc::SOCK_CLOEXEC,
            libc::NETLINK_KOBJECT_UEVENT,
        );
        if fd < 0 {
            eprintln!("uevent: socket failed: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        let mut sa: libc::sockaddr_nl = std::mem::zeroed();
        sa.nl_family = libc::AF_NETLINK as _;
        sa.nl_groups = 1;
        if libc::bind(
            fd,
            &sa as *const _ as _,
            std::mem::size_of::<libc::sockaddr_nl>() as _,
        ) != 0
        {
            eprintln!("uevent: bind failed: {}", std::io::Error::last_os_error());
            libc::close(fd);
            return Ok(1);
        }

        let tv = libc::timeval {
            tv_sec: 15,
            tv_usec: 0,
        };
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &tv as *const _ as _,
            std::mem::size_of::<libc::timeval>() as _,
        );
        let mut out = wlock();
        let mut seen = 0u64;
        let mut buf = [0u8; 8192];
        loop {
            let n = libc::recv(fd, buf.as_mut_ptr() as _, buf.len(), 0);
            if n <= 0 {
                break;
            }
            let n = n as usize;

            let mut first = true;
            for part in buf[..n].split(|&c| c == 0) {
                if part.is_empty() {
                    continue;
                }
                if first {
                    let _ = writeln!(out, "--- {}", String::from_utf8_lossy(part));
                    first = false;
                } else {
                    let _ = writeln!(out, "{}", String::from_utf8_lossy(part));
                }
            }
            seen += 1;
            if count.is_some_and(|c| seen >= c) {
                break;
            }
        }
        libc::close(fd);
    }
    Ok(0)
}

applet!(
    MakedevsApplet,
    "makedevs",
    "Create device nodes from table",
    run_makedevs
);
fn run_makedevs(args: &[OsString]) -> Result<i32> {
    let (mut table, mut root) = (None::<OsString>, String::from("/"));
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" {
            i += 1;
            if i >= args.len() {
                eprintln!("makedevs: -d needs an argument");
                return Ok(1);
            }
            table = Some(args[i].clone());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("makedevs: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else if root == "/" {
            root = lossy(&args[i]);
        } else {
            eprintln!("makedevs: too many arguments");
            return Ok(1);
        }
        i += 1;
    }
    let data = match &table {
        Some(t) => match read_all(t.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("makedevs: can't open '{}': {}", lossy(t), e);
                return Ok(1);
            }
        },
        None => {
            eprintln!("makedevs: needs -d TABLE");
            return Ok(1);
        }
    };
    let mut rc = 0;
    for (ln, raw) in String::from_utf8_lossy(&data).lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 2 {
            eprintln!("makedevs: line {}: bad format", ln + 1);
            rc = 1;
            continue;
        }
        let path = format!(
            "{}{}{}",
            root.trim_end_matches('/'),
            if root.ends_with('/') { "" } else { "/" },
            f[0]
        );
        match f[1] {
            "d" => {
                let mode = f
                    .get(2)
                    .and_then(|s| u32::from_str_radix(s.trim_start_matches("0o"), 8).ok())
                    .unwrap_or(0o755);
                if std::fs::create_dir_all(&path).is_err()
                    || std::fs::set_permissions(
                        &path,
                        std::os::unix::fs::PermissionsExt::from_mode(mode),
                    )
                    .is_err()
                {
                    eprintln!("makedevs: line {}: mkdir failed", ln + 1);
                    rc = 1;
                }
            }
            "f" => {
                let mode = f
                    .get(2)
                    .and_then(|s| u32::from_str_radix(s, 8).ok())
                    .unwrap_or(0o644);
                match File::create(&path) {
                    Ok(_) => {
                        let _ = std::fs::set_permissions(
                            &path,
                            std::os::unix::fs::PermissionsExt::from_mode(mode),
                        );
                    }
                    Err(_) => {
                        eprintln!("makedevs: line {}: create failed", ln + 1);
                        rc = 1;
                    }
                }
            }
            "p" => {
                let c = match std::ffi::CString::new(path.as_bytes()) {
                    Ok(c) => c,
                    Err(_) => {
                        rc = 1;
                        continue;
                    }
                };
                if unsafe { libc::mkfifo(c.as_ptr(), 0o644) } != 0
                    && std::io::Error::last_os_error().kind() != std::io::ErrorKind::AlreadyExists
                {
                    eprintln!("makedevs: line {}: mkfifo failed", ln + 1);
                    rc = 1;
                }
            }
            "s" => {
                if f.len() < 3 {
                    eprintln!("makedevs: line {}: symlink needs target", ln + 1);
                    rc = 1;
                    continue;
                }
                if std::os::unix::fs::symlink(f[2], &path).is_err() {
                    eprintln!("makedevs: line {}: symlink failed", ln + 1);
                    rc = 1;
                }
            }
            "c" | "b" => {
                if f.len() < 8 {
                    eprintln!(
                        "makedevs: line {}: device needs maj min mode uid gid",
                        ln + 1
                    );
                    rc = 1;
                    continue;
                }
                let maj: u32 = f[2].parse().unwrap_or(0);
                let min: u32 = f[3].parse().unwrap_or(0);
                let mode = u32::from_str_radix(f[4], 8).unwrap_or(0o660);
                let kind = if f[1] == "c" {
                    libc::S_IFCHR
                } else {
                    libc::S_IFBLK
                };
                let c = match std::ffi::CString::new(path.as_bytes()) {
                    Ok(c) => c,
                    Err(_) => {
                        rc = 1;
                        continue;
                    }
                };
                if unsafe { libc::mknod(c.as_ptr(), kind | mode, libc::makedev(maj, min)) } != 0 {
                    eprintln!(
                        "makedevs: line {}: mknod failed: {}",
                        ln + 1,
                        std::io::Error::last_os_error()
                    );
                    rc = 1;
                    continue;
                }
                let uid: u32 = f[5].parse().unwrap_or(0);
                let gid: u32 = f[6].parse().unwrap_or(0);
                unsafe {
                    libc::chown(c.as_ptr(), uid, gid);
                }
            }
            t => {
                eprintln!("makedevs: line {}: unknown type '{}'", ln + 1, t);
                rc = 1;
            }
        }
    }
    Ok(rc)
}

applet!(
    SwaplabelApplet,
    "swaplabel",
    "Print or change swap LABEL/UUID",
    run_swaplabel
);
fn run_swaplabel(args: &[OsString]) -> Result<i32> {
    let (mut want_l, mut want_u) = (false, false);
    let mut dev: Option<OsString> = None;
    for a in args {
        let b = ab(a);
        if b == b"-l" || b == b"--label" {
            want_l = true;
        } else if b == b"-U" || b == b"--uuid" {
            want_u = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("swaplabel: invalid option '{}'", lossy(a));
            return Ok(1);
        } else if dev.is_none() {
            dev = Some(a.clone());
        } else {
            eprintln!("swaplabel: too many arguments");
            return Ok(1);
        }
    }
    let dev = match dev {
        Some(d) => d,
        None => {
            eprintln!("swaplabel: needs a device argument");
            return Ok(1);
        }
    };
    let vi = match probe_vol(dev.as_os_str()) {
        Some(v) if v.fstype == "swap" => v,
        _ => {
            eprintln!("swaplabel: '{}' is not a swap device", lossy(&dev));
            return Ok(1);
        }
    };
    let mut out = wlock();
    if want_l && !want_u {
        let _ = writeln!(out, "LABEL: {}", vi.label);
    } else if want_u && !want_l {
        let _ = writeln!(out, "UUID: {}", vi.uuid);
    } else {
        let _ = writeln!(out, "LABEL: {}", vi.label);
        let _ = writeln!(out, "UUID: {}", vi.uuid);
    }
    Ok(0)
}

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
applet!(IpcrmApplet, "ipcrm", "Remove IPC objects", run_ipcrm);
fn run_ipcrm(args: &[OsString]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("ipcrm: needs arguments (use -m/-s/-q ID or -M/-S/-Q KEY)");
        return Ok(1);
    }
    let mut rc = 0;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        let (kind, by_key) = match b {
            b"-m" => (b'm', false),
            b"-s" => (b's', false),
            b"-q" => (b'q', false),
            b"-M" => (b'm', true),
            b"-S" => (b's', true),
            b"-Q" => (b'q', true),
            b"-a" => {
                for id in 0..64 {
                    unsafe {
                        libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut());
                        libc::semctl(id, 0, libc::IPC_RMID);
                        libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut());
                    }
                }
                i += 1;
                continue;
            }
            _ => {
                eprintln!("ipcrm: invalid option '{}'", lossy(&args[i]));
                return Ok(1);
            }
        };
        i += 1;
        if i >= args.len() {
            eprintln!("ipcrm: option needs an argument");
            return Ok(1);
        }
        let v = lossy(&args[i]);
        if let Err(e) = ipc_key_rm(kind, &v, by_key) {
            eprintln!("ipcrm: can't remove {} '{}': {}", lossy(&args[i - 1]), v, e);
            rc = 1;
        }
        i += 1;
    }
    Ok(rc)
}
applet!(IpcsApplet, "ipcs", "Show IPC facilities", run_ipcs);
fn run_ipcs(args: &[OsString]) -> Result<i32> {
    let (mut m, mut s, mut q) = (false, false, false);
    for a in args {
        let b = ab(a);
        if b == b"-a" || b == b"--all" {
            m = true;
            s = true;
            q = true;
        } else if b == b"-m" {
            m = true;
        } else if b == b"-s" {
            s = true;
        } else if b == b"-q" {
            q = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("ipcs: invalid option '{}'", lossy(a));
            return Ok(1);
        }
    }
    if !m && !s && !q {
        m = true;
        s = true;
        q = true;
    }
    let mut out = wlock();
    unsafe {
        if m {
            let _ = writeln!(out, "------ Shared Memory Segments --------");
            let _ = writeln!(out, "{:<8} {:<10} {:<6}", "shmid", "perms", "nattch");
            for id in 0..128 {
                let mut ds: libc::shmid_ds = std::mem::zeroed();
                if libc::shmctl(id, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(
                        out,
                        "{:<8} {:<10o} {:<6}",
                        id,
                        ds.shm_perm.mode & 0o777,
                        ds.shm_nattch
                    );
                }
            }
        }
        if s {
            let _ = writeln!(out, "------ Semaphore Arrays --------");
            let _ = writeln!(out, "{:<8} {:<10}", "semid", "perms");
            for id in 0..128 {
                let mut ds: libc::semid_ds = std::mem::zeroed();

                if libc::semctl(id, 0, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(out, "{:<8} {:<10o}", id, ds.sem_perm.mode & 0o777);
                }
            }
        }
        if q {
            let _ = writeln!(out, "------ Message Queues --------");
            let _ = writeln!(out, "{:<8} {:<10} {:<8}", "msqid", "perms", "messages");
            for id in 0..128 {
                let mut ds: libc::msqid_ds = std::mem::zeroed();
                if libc::msgctl(id, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(
                        out,
                        "{:<8} {:<10o} {:<8}",
                        id,
                        ds.msg_perm.mode & 0o777,
                        ds.msg_qnum
                    );
                }
            }
        }
    }
    Ok(0)
}

fn read_stat() -> Vec<u8> {
    std::fs::read("/proc/stat").unwrap_or_default()
}
fn cpu_line(d: &[u8], prefix: &[u8]) -> Option<Vec<u64>> {
    for line in d.split(|&c| c == b'\n') {
        if line.starts_with(prefix) && line.get(prefix.len()) == Some(&b' ') {
            let mut v = Vec::new();
            for f in line[prefix.len() + 1..].split(|&c| c == b' ' || c == b'\t') {
                if f.is_empty() {
                    continue;
                }
                v.push(String::from_utf8_lossy(f).parse().unwrap_or(0));
            }
            return Some(v);
        }
    }
    None
}
applet!(
    IostatApplet,
    "iostat",
    "Report CPU and I/O statistics",
    run_iostat
);
fn run_iostat(args: &[OsString]) -> Result<i32> {
    let (mut cpu_only, mut dev_only) = (false, false);
    for a in args {
        let b = ab(a);
        if b == b"-c" {
            cpu_only = true;
        } else if b == b"-d" {
            dev_only = true;
        } else if b == b"-k" || b == b"-m" {
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            let mut ok = true;
            for &c in &b[1..] {
                if !matches!(c, b'V' | b'x' | b'z' | b'N' | b'y') {
                    ok = false;
                    break;
                }
            }
            if !ok {
                eprintln!("iostat: invalid option '{}'", lossy(a));
                return Ok(1);
            }
        }
    }
    let st = read_stat();
    let mut out = wlock();
    if !dev_only {
        let c = cpu_line(&st, b"cpu").unwrap_or_default();
        let tot: u64 = c.iter().sum();
        let idle = c.get(3).copied().unwrap_or(0) + c.get(4).copied().unwrap_or(0);
        let busy = tot.saturating_sub(idle);
        let pct = |v: u64| {
            if tot == 0 {
                0.0
            } else {
                v as f64 * 100.0 / tot as f64
            }
        };
        let _ = writeln!(out, "avg-cpu:  %user   %nice %system %iowait  %idle");
        let _ = writeln!(
            out,
            "          {:6.2} {:6.2} {:6.2} {:6.2} {:6.2}",
            pct(c.first().copied().unwrap_or(0)
                + busy.saturating_sub(
                    c.get(2).copied().unwrap_or(0) + c.first().copied().unwrap_or(0)
                )),
            pct(c.get(1).copied().unwrap_or(0)),
            pct(c.get(2).copied().unwrap_or(0)),
            pct(c.get(4).copied().unwrap_or(0)),
            pct(idle)
        );
    }
    if !cpu_only {
        let _ = writeln!(
            out,
            "Device:            tps    kB_read/s    kB_wrtn/s    kB_read    kB_wrtn"
        );
        if let Ok(dd) = std::fs::read("/proc/diskstats") {
            for line in dd.split(|&c| c == b'\n') {
                let f: Vec<&[u8]> = line
                    .split(|&c| c == b' ' || c == b'\t')
                    .filter(|x| !x.is_empty())
                    .collect();
                if f.len() < 14 {
                    continue;
                }
                let nm = String::from_utf8_lossy(f[2]).into_owned();
                let rd: u64 = String::from_utf8_lossy(f[5]).parse().unwrap_or(0);
                let wr: u64 = String::from_utf8_lossy(f[9]).parse().unwrap_or(0);
                let rs: u64 = String::from_utf8_lossy(f[3]).parse().unwrap_or(0);
                let ws: u64 = String::from_utf8_lossy(f[7]).parse().unwrap_or(0);
                let _ = writeln!(
                    out,
                    "{:<16} {:6.2} {:12} {:12} {:9} {:9}",
                    nm,
                    (rs + ws) as f64,
                    rd / 2,
                    wr / 2,
                    rd / 2,
                    wr / 2
                );
            }
        }
    }
    Ok(0)
}
applet!(
    MpstatApplet,
    "mpstat",
    "Report per-CPU statistics",
    run_mpstat
);
fn run_mpstat(args: &[OsString]) -> Result<i32> {
    let mut only: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-P" {
            i += 1;
            if i >= args.len() {
                eprintln!("mpstat: -P needs an argument");
                return Ok(1);
            }
            only = Some(lossy(&args[i]));
        } else if b.starts_with(b"-P") && b.len() > 2 {
            only = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("mpstat: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        }
        i += 1;
    }
    let st = read_stat();
    let mut out = wlock();
    let _ = writeln!(out, "CPU    %usr   %nice    %sys %iowait    %idle");
    for line in st.split(|&c| c == b'\n') {
        if !line.starts_with(b"cpu") {
            continue;
        }
        let is_all = line.starts_with(b"cpu ");
        let id = if is_all {
            "all".to_string()
        } else {
            String::from_utf8_lossy(
                &line[..line.iter().position(|&c| c == b' ').unwrap_or(line.len())],
            )
            .into_owned()
        };
        if let Some(f) = &only {
            if f != "ALL" && id != *f && !(is_all && f == "all") {
                continue;
            }
        }
        let nums: Vec<u64> = line
            .split(|&c| c == b' ' || c == b'\t')
            .skip(1)
            .filter(|x| !x.is_empty())
            .map(|x| String::from_utf8_lossy(x).parse().unwrap_or(0))
            .collect();
        let tot: u64 = nums.iter().sum();
        let pct = |v: u64| {
            if tot == 0 {
                0.0
            } else {
                v as f64 * 100.0 / tot as f64
            }
        };
        let g = |k: usize| nums.get(k).copied().unwrap_or(0);
        let _ = writeln!(
            out,
            "{:<6} {:6.2} {:6.2} {:6.2} {:6.2} {:6.2}",
            id,
            pct(g(0)),
            pct(g(1)),
            pct(g(2)),
            pct(g(4)),
            pct(g(3) + g(4))
        );
    }
    Ok(0)
}
