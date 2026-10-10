use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use super::sha384sum::*;

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
    let mut bits = 224usize;
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
